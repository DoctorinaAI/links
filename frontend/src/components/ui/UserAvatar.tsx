import { createEffect, createSignal, onCleanup, Show } from 'solid-js';
import type { User } from '../../types/auth.types';
import './UserAvatar.css';

interface UserAvatarProps {
  user: User;
  onLogout: () => void;
}

export function UserAvatar(props: UserAvatarProps) {
  const [isMenuOpen, setIsMenuOpen] = createSignal(false);
  const [imageError, setImageError] = createSignal(false);
  const [imageLoaded, setImageLoaded] = createSignal(false);
  let menuRef: HTMLDivElement | undefined;
  let buttonRef: HTMLButtonElement | undefined;

  // Close menu when clicking outside
  const handleClickOutside = (e: MouseEvent) => {
    if (
      menuRef &&
      buttonRef &&
      !menuRef.contains(e.target as Node) &&
      !buttonRef.contains(e.target as Node)
    ) {
      setIsMenuOpen(false);
    }
  };

  createEffect(() => {
    if (isMenuOpen()) {
      document.addEventListener('click', handleClickOutside);
    } else {
      document.removeEventListener('click', handleClickOutside);
    }
  });

  onCleanup(() => {
    document.removeEventListener('click', handleClickOutside);
  });

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen());
  };

  const handleLogout = () => {
    setIsMenuOpen(false);
    props.onLogout();
  };

  const handleImageError = () => {
    setImageError(true);
  };

  const handleImageLoad = () => {
    setImageLoaded(true);
  };

  // Get initials from name for placeholder
  const getInitials = () => {
    const name = props.user.name || props.user.email;
    const parts = name.split(' ');
    if (parts.length >= 2) {
      return (parts[0][0] + parts[1][0]).toUpperCase();
    }
    return name.slice(0, 2).toUpperCase();
  };

  const hasValidPicture = () => {
    return props.user.picture && !imageError();
  };

  return (
    <div class="user-avatar-container">
      <button
        ref={buttonRef}
        class="user-avatar-button"
        onClick={toggleMenu}
        title={props.user.name || props.user.email}
        aria-label="User menu"
        aria-expanded={isMenuOpen()}
      >
        <Show
          when={hasValidPicture()}
          fallback={
            <div class="user-avatar-placeholder">
              {getInitials()}
            </div>
          }
        >
          <img
            src={props.user.picture}
            alt={props.user.name || 'User avatar'}
            class="user-avatar-image"
            classList={{ loaded: imageLoaded() }}
            onError={handleImageError}
            onLoad={handleImageLoad}
            crossOrigin="anonymous"
            referrerPolicy="no-referrer"
          />
        </Show>
      </button>

      <Show when={isMenuOpen()}>
        <div ref={menuRef} class="user-menu">
          <div class="user-menu-header">
            <div class="user-menu-name">{props.user.name}</div>
            <div class="user-menu-email">{props.user.email}</div>
          </div>
          <div class="user-menu-divider"></div>
          <button
            class="user-menu-item"
            onClick={handleLogout}
          >
            <span class="material-icons">logout</span>
            <span>Sign out</span>
          </button>
        </div>
      </Show>
    </div>
  );
}
