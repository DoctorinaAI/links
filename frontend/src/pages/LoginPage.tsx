/**
 * Login Page Component
 * Simple authentication page with Google Sign-In
 */

import { Component, onMount } from 'solid-js';
import { APP_CONFIG } from '../config/app.config';
import { handleGoogleSignIn } from '../services/auth.service';
import { initializeGoogleIdentity, renderGoogleButton } from '../services/google.identity.service';
import { getAuthLoading } from '../stores/auth.store';
import type { GoogleCredentialResponse } from '../types';

/**
 * Login page component
 */
const LoginPage: Component = () => {
  onMount(() => {
    // Initialize Google Identity Services
    initializeGoogleIdentity(
      APP_CONFIG.googleClientId,
      handleGoogleCallback
    );

    // Render Google Sign-In button
    setTimeout(() => {
      renderGoogleButton('google-signin-button', {
        theme: 'outline',
        size: 'large',
        text: 'signin_with',
      });
    }, 100);
  });

  const handleGoogleCallback = async (response: GoogleCredentialResponse) => {
    try {
      await handleGoogleSignIn(response.credential);
    } catch (error) {
      console.error('Sign-in error:', error);
      alert('Failed to sign in. Please try again.');
    }
  };

  return (
    <div style={{
      display: 'flex',
      'align-items': 'center',
      'justify-content': 'center',
      'min-height': '100vh',
      background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    }}>
      <div style={{
        background: 'white',
        padding: '3rem',
        'border-radius': '1rem',
        'box-shadow': '0 20px 60px rgba(0,0,0,0.3)',
        'text-align': 'center',
        'max-width': '400px',
        width: '100%',
      }}>
        <h1 style={{
          'font-size': '2rem',
          'margin-bottom': '0.5rem',
          color: '#667eea',
        }}>
          Links Admin
        </h1>
        <p style={{
          color: '#718096',
          'margin-bottom': '2rem',
        }}>
          Sign in to manage your links
        </p>

        <div
          id="google-signin-button"
          style={{
            display: 'flex',
            'justify-content': 'center',
            'margin-bottom': '1rem',
          }}
        />

        {getAuthLoading()() && (
          <p style={{ color: '#667eea', 'margin-top': '1rem' }}>
            Signing in...
          </p>
        )}
      </div>
    </div>
  );
};

export default LoginPage;
