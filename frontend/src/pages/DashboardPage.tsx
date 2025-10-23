/**
 * Dashboard Page - Manage short links
 */

import { Component, createMemo, createSignal, For, onMount, Show } from 'solid-js';
import Modal from '../components/ui/Modal';
import ShortLinkForm from '../components/ui/ShortLinkForm';
import { UserAvatar } from '../components/ui/UserAvatar';
import {
    createShortLink,
    deleteShortLink,
    getClickStats,
    getShortLinks,
    updateShortLink
} from '../services/api.service';
import { logoutUser } from '../services/auth.service';
import { getUser } from '../stores/auth.store';
import { toastStore } from '../stores/toast.store';
import { ClickStatsResponse, ShortLink } from '../types';
import { canDeleteLink } from '../utils';
import { formatFullDate, formatRelativeDate } from '../utils/date.utils';
import './DashboardPage.css';

type ViewMode = 'grid' | 'table';

const DashboardPage: Component = () => {
  const user = getUser();

  // State
  const [links, setLinks] = createSignal<ShortLink[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [viewMode, setViewMode] = createSignal<ViewMode>(
    (localStorage.getItem('linksViewMode') as ViewMode) || 'grid'
  );
  const [isCreateModalOpen, setIsCreateModalOpen] = createSignal(false);
  const [isEditModalOpen, setIsEditModalOpen] = createSignal(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = createSignal(false);
  const [isStatsModalOpen, setIsStatsModalOpen] = createSignal(false);
  const [isDuplicateModalOpen, setIsDuplicateModalOpen] = createSignal(false);
  const [selectedLink, setSelectedLink] = createSignal<ShortLink | null>(null);
  const [stats, setStats] = createSignal<ClickStatsResponse | null>(null);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [searchQuery, setSearchQuery] = createSignal('');

  // Filtered links based on search query
  const filteredLinks = createMemo(() => {
    const query = searchQuery().toLowerCase().trim();
    if (!query) return links();

    return links().filter(link => {
      // Search in slug
      if (link.slug.toLowerCase().includes(query)) return true;

      // Search in author
      if (link.author.toLowerCase().includes(query)) return true;

      // Search in redirect URL
      if (link.redirect && link.redirect.toLowerCase().includes(query)) return true;

      // Search in parameter keys
      const paramKeys = Object.keys(link.params);
      if (paramKeys.some(key => key.toLowerCase().includes(query))) return true;

      // Search in parameter values
      const paramValues = Object.values(link.params);
      if (paramValues.some(value => value.toLowerCase().includes(query))) return true;

      return false;
    });
  });

  // Save view mode preference
  const toggleViewMode = (mode: ViewMode) => {
    setViewMode(mode);
    localStorage.setItem('linksViewMode', mode);
  };

  // Load links
  const loadLinks = async () => {
    setLoading(true);
    try {
      const response = await getShortLinks();
      if (response.success && response.data) {
        setLinks(response.data.links);
      } else {
        toastStore.showError('Failed to load links');
      }
    } catch (error) {
      console.error('Error loading links:', error);
      toastStore.showError('Failed to load links');
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadLinks();
  });

  // Handlers
  const handleLogout = () => {
    logoutUser();
    toastStore.showInfo('You have been logged out');
  };

  const handleCreate = async (data: {
    slug: string;
    params: Record<string, string>;
    author: string;
    redirect: string | null;
  }) => {
    setIsSubmitting(true);
    try {
      const response = await createShortLink(data);
      if (response.success) {
        toastStore.showSuccess('Link created successfully');
        setIsCreateModalOpen(false);
        await loadLinks();
      } else {
        toastStore.showError(response.error?.message || 'Failed to create link');
      }
    } catch (error) {
      console.error('Error creating link:', error);
      toastStore.showError('Failed to create link');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDuplicate = async (data: {
    slug: string;
    params: Record<string, string>;
    author: string;
    redirect: string | null;
  }) => {
    setIsSubmitting(true);
    try {
      const response = await createShortLink(data);
      if (response.success) {
        toastStore.showSuccess('Link duplicated successfully');
        setIsDuplicateModalOpen(false);
        setSelectedLink(null);
        await loadLinks();
      } else {
        toastStore.showError(response.error?.message || 'Failed to duplicate link');
      }
    } catch (error) {
      console.error('Error duplicating link:', error);
      toastStore.showError('Failed to duplicate link');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleEdit = async (data: {
    slug: string;
    params: Record<string, string>;
    author: string;
    redirect: string | null;
  }) => {
    const link = selectedLink();
    if (!link) return;

    setIsSubmitting(true);
    try {
      const response = await updateShortLink(link.slug, data);
      if (response.success) {
        toastStore.showSuccess('Link updated successfully');
        setIsEditModalOpen(false);
        setSelectedLink(null);
        await loadLinks();
      } else {
        toastStore.showError(response.error?.message || 'Failed to update link');
      }
    } catch (error) {
      console.error('Error updating link:', error);
      toastStore.showError('Failed to update link');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async () => {
    const link = selectedLink();
    if (!link) return;

    setIsSubmitting(true);
    try {
      const response = await deleteShortLink(link.slug);
      if (response.success) {
        toastStore.showSuccess('Link deleted successfully');
        setIsDeleteModalOpen(false);
        setSelectedLink(null);
        await loadLinks();
      } else {
        toastStore.showError(response.error?.message || 'Failed to delete link');
      }
    } catch (error) {
      console.error('Error deleting link:', error);
      toastStore.showError('Failed to delete link');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleViewStats = async (link: ShortLink) => {
    setSelectedLink(link);
    setIsStatsModalOpen(true);
    try {
      const response = await getClickStats(link.slug);
      if (response.success && response.data) {
        setStats(response.data);
      } else {
        toastStore.showError('Failed to load statistics');
      }
    } catch (error) {
      console.error('Error loading stats:', error);
      toastStore.showError('Failed to load statistics');
    }
  };

  const openEditModal = (link: ShortLink) => {
    setSelectedLink(link);
    setIsEditModalOpen(true);
  };

  const openDeleteModal = (link: ShortLink) => {
    setSelectedLink(link);
    setIsDeleteModalOpen(true);
  };

  const openDuplicateModal = (link: ShortLink) => {
    setSelectedLink(link);
    setIsDuplicateModalOpen(true);
  };

  const handleCardClick = (link: ShortLink, e: MouseEvent) => {
    // Don't open if clicking on a button or link
    const target = e.target as HTMLElement;
    if (target.closest('button') || target.closest('a')) {
      return;
    }
    handleViewStats(link);
  };

  const getFullUrl = (slug: string) => {
    return `${window.location.origin}/s/${slug}`;
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toastStore.showSuccess('Copied to clipboard');
  };

  return (
    <div class="dashboard">
      {/* Header */}
      <div class="dashboard-header">
        <div>
          <h1 class="dashboard-title">Dashboard</h1>
          {/* <p class="dashboard-subtitle">
            Welcome, {user()?.name || 'User'}
          </p> */}
        </div>
        <div class="dashboard-actions">
          <Show when={user()}>
            {(currentUser) => (
              <UserAvatar
                user={currentUser()}
                onLogout={handleLogout}
              />
            )}
          </Show>
        </div>
      </div>

      {/* Links Table */}
      <div class="dashboard-card">
        <Show
          when={!loading()}
          fallback={
            <div class="loading-state">
              <div class="spinner"></div>
              <p>Loading links...</p>
            </div>
          }
        >
          <Show
            when={links().length > 0}
            fallback={
              <div class="empty-state">
                <svg
                  class="empty-state-icon"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
                  />
                </svg>
                <h3>No links yet</h3>
                <p>Create your first link to get started</p>
              </div>
            }
          >
            {/* View Controls */}
            <div class="view-controls">
              <div class="links-count">
                {links().length} {links().length === 1 ? 'link' : 'links'}
              </div>
              <div class="search-box">
                <span class="material-icons search-icon">search</span>
                <input
                  type="text"
                  class="search-input"
                  placeholder="Search links..."
                  value={searchQuery()}
                  onInput={(e) => setSearchQuery(e.currentTarget.value)}
                />
                <Show when={searchQuery()}>
                  <button
                    class="search-clear"
                    onClick={() => setSearchQuery('')}
                    title="Clear search"
                  >
                    <span class="material-icons">close</span>
                  </button>
                </Show>
              </div>
              <div class="view-toggle">
                <button
                  class={`view-toggle-btn ${viewMode() === 'grid' ? 'active' : ''}`}
                  onClick={() => toggleViewMode('grid')}
                  title="Grid view"
                >
                  <span class="material-icons">grid_view</span>
                  Grid
                </button>
                <button
                  class={`view-toggle-btn ${viewMode() === 'table' ? 'active' : ''}`}
                  onClick={() => toggleViewMode('table')}
                  title="Table view"
                >
                  <span class="material-icons">table_rows</span>
                  Table
                </button>
              </div>
            </div>

            {/* No Results */}
            <Show when={filteredLinks().length === 0 && searchQuery()}>
              <div class="empty-state">
                <span class="material-icons empty-state-icon" style="font-size: 64px; color: #9CA3AF;">search_off</span>
                <h3>No results found</h3>
                <p>No links match your search: "{searchQuery()}"</p>
                <button
                  class="btn btn-secondary"
                  onClick={() => setSearchQuery('')}
                >
                  Clear search
                </button>
              </div>
            </Show>

            {/* Grid View */}
            <Show when={viewMode() === 'grid' && filteredLinks().length > 0}>
              <div class="links-grid">
                <For each={filteredLinks()}>
                  {(link) => (
                    <div class="link-card" onClick={(e) => handleCardClick(link, e)}>
                      <div class="link-card-header">
                        <div class="link-card-slug">
                          <div class="slug-label">Slug</div>
                          <div class="slug-value">
                            <span class="slug-text">{link.slug}</span>
                            <Show when={link.redirect}>
                            <div class="link-card-params">
                                <span class="meta-icon">🔗</span>
                            </div>
                            </Show>
                            <Show when={Object.keys(link.params).length > 0}>
                              <span class="slug-badge">
                                {Object.keys(link.params).length}
                              </span>
                            </Show>
                          </div>
                        </div>
                      </div>

                      <Show when={link.description}>
                        <div class="link-card-description">
                          <p>{link.description}</p>
                        </div>
                      </Show>

                      <div class="link-card-footer">
                        <div class="date-info" title={formatFullDate(link.updated_at)}>
                          {formatRelativeDate(link.updated_at)}
                        </div>
                        <div class="action-buttons-compact">
                          <button
                            class="btn-action"
                            onClick={() => copyToClipboard(getFullUrl(link.slug))}
                            title="Copy full URL"
                          >
                            <span class="material-icons">content_copy</span>
                          </button>
                          <Show when={link.description}>
                            <button
                              class="btn-action"
                              onClick={() => copyToClipboard(link.description!)}
                              title="Copy description"
                            >
                              📋
                            </button>
                          </Show>
                          <button
                            class="btn-action"
                            onClick={() => openDuplicateModal(link)}
                            title="Duplicate link"
                          >
                            <span class="material-icons">file_copy</span>
                          </button>
                          <button
                            class="btn-action"
                            onClick={() => openEditModal(link)}
                            title="Edit link"
                          >
                            <span class="material-icons">edit</span>
                          </button>
                          <button
                            class="btn-action"
                            onClick={() => openDeleteModal(link)}
                            disabled={!canDeleteLink(link.created_at)}
                            title={
                              canDeleteLink(link.created_at)
                                ? 'Delete link'
                                : 'Cannot delete links older than 30 minutes'
                            }
                          >
                            <span class="material-icons">delete</span>
                          </button>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* Table View */}
            <Show when={viewMode() === 'table' && filteredLinks().length > 0}>
              <div class="table-container">
                <table class="links-table">
                  <thead>
                    <tr>
                      <th style="text-align: left;">Slug</th>
                      <th>Description</th>
                      <th>Params</th>
                      <th>Updated</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={filteredLinks()}>
                      {(link) => (
                        <tr onClick={(e) => handleCardClick(link, e)}>
                          <td>
                            <code class="slug-code" title={link.slug}>{link.slug}</code>
                          </td>
                          <td>
                            <Show
                              when={link.description}
                              fallback={<span class="text-muted">—</span>}
                            >
                              <div class="description-preview" title={link.description!}>
                                {link.description!.length > 50
                                  ? link.description!.substring(0, 50) + '...'
                                  : link.description}
                              </div>
                            </Show>
                          </td>
                          <td>
                            <Show
                              when={Object.keys(link.params).length > 0}
                              fallback={<span class="text-muted">—</span>}
                            >
                              <div class="params-preview">
                                {Object.keys(link.params).length}
                              </div>
                            </Show>
                          </td>
                          <td class="text-muted text-sm" title={formatFullDate(link.updated_at)}>
                            {formatRelativeDate(link.updated_at)}
                          </td>
                          <td>
                            <div class="action-buttons">
                              <button
                                class="btn-action"
                                onClick={() => copyToClipboard(getFullUrl(link.slug))}
                                title="Copy full URL"
                              >
                                <span class="material-icons">content_copy</span>
                              </button>
                              <Show when={link.description}>
                                <button
                                  class="btn-action"
                                  onClick={() => copyToClipboard(link.description!)}
                                  title="Copy description"
                                >
                                  📋
                                </button>
                              </Show>
                              <button
                                class="btn-action"
                                onClick={() => openDuplicateModal(link)}
                                title="Duplicate link"
                              >
                                <span class="material-icons">file_copy</span>
                              </button>
                              <button
                                class="btn-action"
                                onClick={() => openEditModal(link)}
                                title="Edit link"
                              >
                                <span class="material-icons">edit</span>
                              </button>
                              <button
                                class="btn-action"
                                onClick={() => openDeleteModal(link)}
                                disabled={!canDeleteLink(link.created_at)}
                                title={
                                  canDeleteLink(link.created_at)
                                    ? 'Delete link'
                                    : 'Cannot delete links older than 30 minutes'
                                }
                              >
                                <span class="material-icons">delete</span>
                              </button>
                            </div>
                          </td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>
            </Show>
          </Show>
        </Show>
      </div>

      {/* Floating Action Button */}
      <button
        class="fab"
        onClick={() => setIsCreateModalOpen(true)}
        title="Create Link"
      >
        <span class="material-icons">add</span>
      </button>

      {/* Create Modal */}
      <Modal
        isOpen={isCreateModalOpen()}
        onClose={() => setIsCreateModalOpen(false)}
        title="Create Link"
        maxWidth="700px"
      >
        <ShortLinkForm
          author={user()?.email || ''}
          onSubmit={handleCreate}
          onCancel={() => setIsCreateModalOpen(false)}
          isSubmitting={isSubmitting()}
        />
      </Modal>

      {/* Edit Modal */}
      <Modal
        isOpen={isEditModalOpen()}
        onClose={() => {
          setIsEditModalOpen(false);
          setSelectedLink(null);
        }}
        title="Edit Link"
        maxWidth="700px"
      >
        <Show when={selectedLink()}>
          <ShortLinkForm
            link={selectedLink()!}
            author={user()?.email || ''}
            onSubmit={handleEdit}
            onCancel={() => {
              setIsEditModalOpen(false);
              setSelectedLink(null);
            }}
            isSubmitting={isSubmitting()}
          />
        </Show>
      </Modal>

      {/* Duplicate Modal */}
      <Modal
        isOpen={isDuplicateModalOpen()}
        onClose={() => {
          setIsDuplicateModalOpen(false);
          setSelectedLink(null);
        }}
        title="Duplicate Link"
        maxWidth="700px"
      >
        <Show when={selectedLink()}>
          <ShortLinkForm
            author={user()?.email || ''}
            initialRedirect={selectedLink()!.redirect}
            initialDescription={selectedLink()!.description}
            initialParams={selectedLink()!.params}
            onSubmit={handleDuplicate}
            onCancel={() => {
              setIsDuplicateModalOpen(false);
              setSelectedLink(null);
            }}
            isSubmitting={isSubmitting()}
          />
        </Show>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        isOpen={isDeleteModalOpen()}
        onClose={() => {
          setIsDeleteModalOpen(false);
          setSelectedLink(null);
        }}
        title="Delete Link"
        maxWidth="500px"
      >
        <div class="delete-modal">
          <p class="delete-message">
            Are you sure you want to delete the link <code>{selectedLink()?.slug}</code>?
          </p>
          <p class="delete-warning">
            This action cannot be undone.
            <br />
            All analytics data for this link will be lost.
            <br />
            Current users may still have access to the link.
            <br />
            Are you sure you want to proceed?
          </p>
          <div class="delete-actions">
            <button
              class="btn btn-secondary"
              onClick={() => {
                setIsDeleteModalOpen(false);
                setSelectedLink(null);
              }}
              disabled={isSubmitting()}
            >
              Cancel
            </button>
            <button
              class="btn btn-danger"
              onClick={handleDelete}
              disabled={isSubmitting()}
            >
              {isSubmitting() ? 'Deleting...' : 'Delete Link'}
            </button>
          </div>
        </div>
      </Modal>

      {/* Statistics Modal */}
      <Modal
        isOpen={isStatsModalOpen()}
        onClose={() => {
          setIsStatsModalOpen(false);
          setSelectedLink(null);
          setStats(null);
        }}
        title={`Statistics: ${selectedLink()?.slug || ''}`}
        maxWidth="600px"
      >
        <div class="stats-modal">
          <Show
            when={stats()}
            fallback={
              <div class="loading-state">
                <div class="spinner"></div>
                <p>Loading statistics...</p>
              </div>
            }
          >
            <div class="stats-summary">
              <div class="stat-card">
                <div class="stat-value">{stats()!.total_clicks}</div>
                <div class="stat-label">Total Clicks</div>
              </div>
            </div>

            <Show when={selectedLink()}>
              <div class="stats-section">
                <h3 class="stats-section-title">Link Details</h3>
                <dl class="details-list">
                  <dt>Slug:</dt>
                  <dd>
                    <code>{selectedLink()!.slug}</code>
                  </dd>
                  <dt>Full URL:</dt>
                  <dd>
                    <code>{getFullUrl(selectedLink()!.slug)}</code>
                  </dd>
                  <dt>Author:</dt>
                  <dd>{selectedLink()!.author}</dd>
                  <Show when={selectedLink()!.description}>
                    <dt>Description:</dt>
                    <dd>
                      <div class="description-display">
                        <p>{selectedLink()!.description}</p>
                        <button
                          class="btn btn-icon"
                          onClick={() => copyToClipboard(selectedLink()!.description!)}
                          title="Copy description"
                        >
                          📋
                        </button>
                      </div>
                    </dd>
                  </Show>
                  <Show when={selectedLink()!.redirect}>
                    <dt>Redirects to:</dt>
                    <dd>
                      <a
                        href={selectedLink()!.redirect!}
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        {selectedLink()!.redirect}
                      </a>
                    </dd>
                  </Show>
                  <dt>Created:</dt>
                  <dd>{formatFullDate(selectedLink()!.created_at)}</dd>
                  <dt>Updated:</dt>
                  <dd>{formatFullDate(selectedLink()!.updated_at)}</dd>
                  <Show when={Object.keys(selectedLink()!.params).length > 0}>
                    <dt>Parameters:</dt>
                    <dd>
                      <div class="params-list-stats">
                        <For each={Object.entries(selectedLink()!.params)}>
                          {([key, value]) => (
                            <div class="param-item">
                              <code class="param-key">{key}</code>
                              <span class="param-separator">=</span>
                              <code class="param-value">{value}</code>
                            </div>
                          )}
                        </For>
                      </div>
                    </dd>
                  </Show>
                </dl>
              </div>
            </Show>

            <Show when={stats()!.recent_clicks.length > 0}>
              <div class="stats-section">
                <h3 class="stats-section-title">Recent Clicks</h3>
                <ul class="clicks-list">
                  <For each={stats()!.recent_clicks}>
                    {(click) => (
                      <li class="click-item">{formatFullDate(click)}</li>
                    )}
                  </For>
                </ul>
              </div>
            </Show>
          </Show>
        </div>
      </Modal>
    </div>
  );
};

export default DashboardPage;
