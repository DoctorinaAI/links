/**
 * Dashboard Page - Manage short links
 */

import { Component } from 'solid-js';
import { logoutUser } from '../services/auth.service';
import { getUser } from '../stores/auth.store';

const DashboardPage: Component = () => {
  const user = getUser();

  const handleLogout = () => {
    logoutUser();
  };

  return (
    <div style={{ padding: '2rem' }}>
      <div style={{
        display: 'flex',
        'justify-content': 'space-between',
        'align-items': 'center',
        'margin-bottom': '2rem'
      }}>
        <div>
          <h1 style={{ margin: 0 }}>Dashboard</h1>
          <p style={{ margin: '0.5rem 0 0 0', color: '#718096' }}>
            Welcome, {user()?.name || 'User'}
          </p>
        </div>
        <button
          onClick={handleLogout}
          style={{
            padding: '0.5rem 1rem',
            background: '#EF4444',
            color: 'white',
            border: 'none',
            'border-radius': '0.5rem',
            cursor: 'pointer',
            'font-weight': '600',
          }}
        >
          Logout
        </button>
      </div>

      <div style={{
        background: 'white',
        padding: '2rem',
        'border-radius': '1rem',
        'box-shadow': '0 1px 3px rgba(0,0,0,0.1)',
      }}>
        <h2>Manage your short links</h2>
        <p style={{ color: '#718096' }}>
          Link management interface will be here
        </p>
      </div>
    </div>
  );
};

export default DashboardPage;
