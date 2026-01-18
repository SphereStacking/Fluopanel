import { computed, ref, onUnmounted, type ComputedRef } from 'vue'
import { Octokit } from '@octokit/rest'
import { createSharedStore } from '@arcana/core'

// Types
export interface GitHubItem {
  id: number
  number: number
  title: string
  state: string
  html_url: string
  repository_url: string
  created_at: string
  updated_at: string
  labels: Array<{ name: string; color: string }>
  pull_request?: { merged_at: string | null }
}

export interface GitHubNotification {
  id: string
  subject: {
    title: string
    type: string
    url: string | null
  }
  repository: {
    full_name: string
    html_url: string
  }
  reason: string
  unread: boolean
  updated_at: string
}

// Sectioned data structures
export interface IssuesSections {
  opened: GitHubItem[]
  assigned: GitHubItem[]
  recentlyClosed: GitHubItem[]
}

export interface PRsSections {
  opened: GitHubItem[]
  assigned: GitHubItem[]
  reviewRequested: GitHubItem[]
  recentlyClosed: GitHubItem[]
}

export interface GitHubState {
  issues: IssuesSections
  prs: PRsSections
  notifications: GitHubNotification[]
  loading: boolean
  error: string | null
  lastFetched: number | null
}

/**
 * Convert GitHub API URL to HTML URL
 */
function apiUrlToHtmlUrl(apiUrl: string | null, repoHtmlUrl: string): string {
  if (!apiUrl) return repoHtmlUrl
  const match = apiUrl.match(/api\.github\.com\/repos\/(.+)/)
  if (match) {
    return `https://github.com/${match[1]}`
  }
  return repoHtmlUrl
}

/**
 * Extract repo name from repository URL
 */
export function extractRepoName(repositoryUrl: string): string {
  const match = repositoryUrl.match(/repos\/(.+)$/)
  return match ? match[1] : repositoryUrl
}

/**
 * Format date to relative time
 */
export function formatDate(dateString: string): string {
  const date = new Date(dateString)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 60) return `${diffMins}m`
  if (diffHours < 24) return `${diffHours}h`
  if (diffDays < 7) return `${diffDays}d`
  return date.toLocaleDateString('en', { month: 'short', day: 'numeric' })
}

// Store key
const STORE_KEY = 'github'

// Create shared store instance
const store = createSharedStore<GitHubState>(STORE_KEY)

// Singleton Octokit instance
const token = import.meta.env.VITE_GITHUB_TOKEN as string | undefined
const octokit = token ? new Octokit({ auth: token }) : null

// Polling state (module level)
let pollIntervalId: ReturnType<typeof setInterval> | null = null
let pollingRefCount = 0

// Cache duration (1 minute)
const CACHE_DURATION = 60 * 1000

// Default state
const defaultState: GitHubState = {
  issues: {
    opened: [],
    assigned: [],
    recentlyClosed: [],
  },
  prs: {
    opened: [],
    assigned: [],
    reviewRequested: [],
    recentlyClosed: [],
  },
  notifications: [],
  loading: false,
  error: null,
  lastFetched: null,
}

export interface UseGitHubReturn {
  // Sectioned data
  issues: ComputedRef<IssuesSections>
  prs: ComputedRef<PRsSections>
  notifications: ComputedRef<GitHubNotification[]>
  loading: ComputedRef<boolean>
  error: ComputedRef<string | null>

  // Counts for bar display (total open items)
  issueCount: ComputedRef<number>
  prCount: ComputedRef<number>
  notificationCount: ComputedRef<number>

  // Token configured check
  isConfigured: ComputedRef<boolean>

  // Actions
  fetch: () => Promise<void>
  fetchIfStale: () => Promise<void>
  refresh: () => Promise<void>
  markNotificationRead: (id: string) => Promise<void>
  getNotificationUrl: (notification: GitHubNotification) => string

  // Polling control
  startPolling: () => void
  stopPolling: () => void
}

/**
 * Vue composable for GitHub data management.
 * Uses SharedStore for cross-window state synchronization.
 */
export function useGitHub(): UseGitHubReturn {
  // Local reactive state that syncs with SharedStore
  const state = ref<GitHubState>(defaultState)

  // Initialize: load from store
  store.get().then((value) => {
    if (value) {
      state.value = value
    }
  })

  // Subscribe to updates from other windows
  const unsubscribe = store.subscribe((value) => {
    state.value = value
  })

  // Cleanup on component unmount
  onUnmounted(() => {
    unsubscribe()
  })

  // Fetch data from GitHub API
  async function fetchData(): Promise<void> {
    const currentStoreValue = await store.get()
    const baseState = currentStoreValue || state.value

    if (!octokit) {
      const newState = {
        ...baseState,
        error: 'Token not configured',
        loading: false,
      }
      state.value = newState
      await store.set(newState)
      return
    }

    // Set loading state (preserve existing data)
    const loadingState = { ...baseState, loading: true, error: null }
    state.value = loadingState
    await store.set(loadingState)

    try {
      // Fetch all sections in parallel
      const [
        issuesOpened,
        issuesAssigned,
        issuesClosed,
        prsOpened,
        prsAssigned,
        prsReviewRequested,
        prsClosed,
        notifications,
      ] = await Promise.all([
        // Issues
        octokit.search.issuesAndPullRequests({
          q: 'is:issue is:open author:@me',
          per_page: 10,
          sort: 'updated',
        }),
        octokit.search.issuesAndPullRequests({
          q: 'is:issue is:open assignee:@me',
          per_page: 10,
          sort: 'updated',
        }),
        octokit.search.issuesAndPullRequests({
          q: 'is:issue is:closed author:@me',
          per_page: 10,
          sort: 'updated',
        }),
        // PRs
        octokit.search.issuesAndPullRequests({
          q: 'is:pr is:open author:@me',
          per_page: 10,
          sort: 'updated',
        }),
        octokit.search.issuesAndPullRequests({
          q: 'is:pr is:open assignee:@me',
          per_page: 10,
          sort: 'updated',
        }),
        octokit.search.issuesAndPullRequests({
          q: 'is:pr is:open review-requested:@me',
          per_page: 10,
          sort: 'updated',
        }),
        octokit.search.issuesAndPullRequests({
          q: 'is:pr is:closed author:@me',
          per_page: 10,
          sort: 'updated',
        }),
        // Notifications
        octokit.activity.listNotificationsForAuthenticatedUser({
          per_page: 20,
        }),
      ])

      const newState: GitHubState = {
        issues: {
          opened: issuesOpened.data.items as GitHubItem[],
          assigned: issuesAssigned.data.items as GitHubItem[],
          recentlyClosed: issuesClosed.data.items as GitHubItem[],
        },
        prs: {
          opened: prsOpened.data.items as GitHubItem[],
          assigned: prsAssigned.data.items as GitHubItem[],
          reviewRequested: prsReviewRequested.data.items as GitHubItem[],
          recentlyClosed: prsClosed.data.items as GitHubItem[],
        },
        notifications: notifications.data as GitHubNotification[],
        loading: false,
        error: null,
        lastFetched: Date.now(),
      }

      state.value = newState
      await store.set(newState)
    } catch (e) {
      console.error('Failed to fetch GitHub data:', e)
      const errorState = {
        ...state.value,
        loading: false,
        error: 'Failed to fetch',
      }
      state.value = errorState
      await store.set(errorState)
    }
  }

  // Fetch if cache is stale
  async function fetchIfStale(): Promise<void> {
    const storeValue = await store.get()
    if (storeValue) {
      state.value = storeValue
      if (storeValue.lastFetched && Date.now() - storeValue.lastFetched <= CACHE_DURATION) {
        return
      }
    }
    await fetchData()
  }

  // Force refresh
  async function refresh(): Promise<void> {
    await fetchData()
  }

  // Mark notification as read
  async function markNotificationRead(notificationId: string): Promise<void> {
    if (!octokit) return

    try {
      await octokit.activity.markThreadAsRead({
        thread_id: parseInt(notificationId),
      })

      const newState = {
        ...state.value,
        notifications: state.value.notifications.map((n) =>
          n.id === notificationId ? { ...n, unread: false } : n
        ),
      }
      state.value = newState
      await store.set(newState)
    } catch {
      // Ignore errors
    }
  }

  // Get notification HTML URL
  function getNotificationUrl(notification: GitHubNotification): string {
    return apiUrlToHtmlUrl(notification.subject.url, notification.repository.html_url)
  }

  // Start polling (5 minute interval)
  function startPolling(): void {
    pollingRefCount++
    if (pollIntervalId) return
    fetchIfStale()
    pollIntervalId = setInterval(() => {
      fetchData()
    }, 5 * 60 * 1000)
  }

  // Stop polling
  function stopPolling(): void {
    pollingRefCount--
    if (pollingRefCount <= 0 && pollIntervalId) {
      clearInterval(pollIntervalId)
      pollIntervalId = null
      pollingRefCount = 0
    }
  }

  return {
    // Sectioned data
    issues: computed(() => state.value.issues),
    prs: computed(() => state.value.prs),
    notifications: computed(() => state.value.notifications),
    loading: computed(() => state.value.loading),
    error: computed(() => state.value.error),

    // Counts for bar display (unique open items)
    issueCount: computed(() => {
      const { opened, assigned } = state.value.issues
      const ids = new Set([...opened.map((i) => i.id), ...assigned.map((i) => i.id)])
      return ids.size
    }),
    prCount: computed(() => {
      const { opened, assigned, reviewRequested } = state.value.prs
      const ids = new Set([
        ...opened.map((i) => i.id),
        ...assigned.map((i) => i.id),
        ...reviewRequested.map((i) => i.id),
      ])
      return ids.size
    }),
    notificationCount: computed(() => state.value.notifications.filter((n) => n.unread).length),

    // Token configured check
    isConfigured: computed(() => !!octokit),

    // Actions
    fetch: fetchData,
    fetchIfStale,
    refresh,
    markNotificationRead,
    getNotificationUrl,

    // Polling control
    startPolling,
    stopPolling,
  }
}
