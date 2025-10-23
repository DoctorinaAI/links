/**
 * Dashboard Page - Manage short links
 */

import { Component, createSignal, For, onMount, Show } from 'solid-js';
import Modal from '../components/ui/Modal';
import ShortLinkForm from '../components/ui/ShortLinkForm';
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
import { canDeleteLink } from '../utils/validation.utils';
import './DashboardPage.css';

const DashboardPage: Component = () => {
  const user = getUser();

  // State
  const [links, setLinks] = createSignal<ShortLink[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [isCreateModalOpen, setIsCreateModalOpen] = createSignal(false);
  const [isEditModalOpen, setIsEditModalOpen] = createSignal(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = createSignal(false);
  const [isStatsModalOpen, setIsStatsModalOpen] = createSignal(false);
  const [selectedLink, setSelectedLink] = createSignal<ShortLink | null>(null);
  const [stats, setStats] = createSignal<ClickStatsResponse | null>(null);
  const [isSubmitting, setIsSubmitting] = createSignal(false);

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

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  const getFullUrl = (slug: string) => {
    return `${window.location.origin}/${slug}`;
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
          <h1 class="dashboard-title">Short Links Dashboard</h1>
          <p class="dashboard-subtitle">
            Welcome, {user()?.name || 'User'}
          </p>
        </div>
        <div class="dashboard-actions">
          <button
            class="btn btn-primary"
            onClick={() => setIsCreateModalOpen(true)}
          >
            + Create Link
          </button>
          <button
            class="btn btn-danger"
            onClick={handleLogout}
          >
            Logout
          </button>
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
                <p>Create your first short link to get started</p>
                <button
                  class="btn btn-primary"
                  onClick={() => setIsCreateModalOpen(true)}
                >
                  Create Link
                </button>
              </div>
            }
          >
            <div class="table-container">
              <table class="links-table">
                <thead>
                  <tr>
                    <th>Slug</th>
                    <th>Redirect</th>
                    <th>Parameters</th>
                    <th>Author</th>
                    <th>Created</th>
                    <th>Updated</th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={links()}>
                    {(link) => (
                      <tr>
                        <td>
                          <div class="slug-cell">
                            <code class="slug-code">{link.slug}</code>
                            <button
                              class="btn-icon-sm"
                              onClick={() => copyToClipboard(getFullUrl(link.slug))}
                              title="Copy full URL"
                            >
                              <svg
                                width="16"
                                height="16"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                              >
                                <path
                                  stroke-linecap="round"
                                  stroke-linejoin="round"
                                  stroke-width="2"
                                  d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                                />
                              </svg>
                            </button>
                          </div>
                        </td>
                        <td>
                          <Show
                            when={link.redirect}
                            fallback={
                              <span class="text-muted">Landing page</span>
                            }
                          >
                            <a
                              href={link.redirect!}
                              target="_blank"
                              rel="noopener noreferrer"
                              class="redirect-link"
                            >
                              {link.redirect}
                            </a>
                          </Show>
                        </td>
                        <td>
                          <Show
                            when={Object.keys(link.params).length > 0}
                            fallback={
                              <span class="text-muted">None</span>
                            }
                          >
                            <div class="params-preview">
                              {Object.keys(link.params).length} parameter(s)
                            </div>
                          </Show>
                        </td>
                        <td>{link.author}</td>
                        <td class="text-muted text-sm">{formatDate(link.created_at)}</td>
                        <td class="text-muted text-sm">{formatDate(link.updated_at)}</td>
                        <td>
                          <div class="action-buttons">
                            <button
                              class="btn-action"
                              onClick={() => handleViewStats(link)}
                              title="View statistics"
                            >
                              📊
                            </button>
                            <button
                              class="btn-action"
                              onClick={() => openEditModal(link)}
                              title="Edit link"
                            >
                              ✏️
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
                              🗑️
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
      </div>

      {/* Create Modal */}
      <Modal
        isOpen={isCreateModalOpen()}
        onClose={() => setIsCreateModalOpen(false)}
        title="Create Short Link"
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
        title="Edit Short Link"
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

      {/* Delete Confirmation Modal */}
      <Modal
        isOpen={isDeleteModalOpen()}
        onClose={() => {
          setIsDeleteModalOpen(false);
          setSelectedLink(null);
        }}
        title="Delete Short Link"
        maxWidth="500px"
      >
        <div class="delete-modal">
          <p class="delete-message">
            Are you sure you want to delete the link <code>{selectedLink()?.slug}</code>?
          </p>
          <p class="delete-warning">
            This action cannot be undone. All analytics data for this link will be lost.
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

            <Show when={stats()!.recent_clicks.length > 0}>
              <div class="stats-section">
                <h3 class="stats-section-title">Recent Clicks</h3>
                <ul class="clicks-list">
                  <For each={stats()!.recent_clicks}>
                    {(click) => (
                      <li class="click-item">{formatDate(click)}</li>
                    )}
                  </For>
                </ul>
              </div>
            </Show>

            <Show when={selectedLink()}>
              <div class="stats-section">
                <h3 class="stats-section-title">Link Details</h3>
                <dl class="details-list">
                  <dt>Full URL:</dt>
                  <dd>
                    <code>{getFullUrl(selectedLink()!.slug)}</code>
                  </dd>
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
          </Show>
        </div>
      </Modal>
    </div>
  );
};

export default DashboardPage;
