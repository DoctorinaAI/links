/**
 * Login Page Component
 * Simple authentication page with Google Sign-In
 */

import { Component, onMount } from 'solid-js';
import { APP_CONFIG } from '../config/app.config';
import { handleGoogleSignIn } from '../services/auth.service';
import { initializeGoogleIdentity, renderGoogleButton } from '../services/google.identity.service';
import { getAuthLoading } from '../stores/auth.store';
import { toastStore } from '../stores/toast.store';
import type { GoogleCredentialResponse } from '../types';
import './LoginPage.css';

/**
 * Login page component
 */
const LoginPage: Component = () => {
  onMount(async () => {
    // Initialize Google Identity Services (wait for script to load)
    await initializeGoogleIdentity(
      APP_CONFIG.googleClientId,
      handleGoogleCallback
    );

    // Render Google Sign-In button after initialization
    renderGoogleButton('google-signin-button', {
      theme: 'outline',
      size: 'large',
      text: 'signin_with',
    });
  });

  const handleGoogleCallback = async (response: GoogleCredentialResponse) => {
    try {
      await handleGoogleSignIn(response.credential);
      toastStore.showSuccess('Successfully signed in!');
    } catch (error) {
      console.error('Sign-in error:', error);
      toastStore.showError('Failed to sign in. Please try again.');
    }
  };

  return (
    <div class="login-container">
      <div class="login-card">
        {/* Decorative corner accent */}
        <div class="login-card__accent" />

        {/* Logo/Icon with subtle animation */}
        <div class="login-card__icon">
          🔗
        </div>

        <h1 class="login-card__title">
          Links Admin
        </h1>

        <p class="login-card__description">
          Sign in with your Google account to manage and track your links
        </p>

        <div
          id="google-signin-button"
          class="login-card__button-container"
        />

        {getAuthLoading()() && (
          <div class="login-card__loading">
            <div class="login-card__spinner" />
            <span>Signing you in...</span>
          </div>
        )}

        {/* Footer */}

        {/* <div class="login-card__footer">
          <p class="login-card__footer-text">
            By signing in, you agree to our{' '}
            <span class="login-card__footer-link">Terms of Service</span>
            {' '}and{' '}
            <span class="login-card__footer-link">Privacy Policy</span>
          </p>
        </div> */}
      </div>
    </div>
  );
};

export default LoginPage;
