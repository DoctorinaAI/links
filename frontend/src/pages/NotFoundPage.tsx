/**
 * Not Found Page - 404 error page
 */

import { A } from '@solidjs/router';
import { Component } from 'solid-js';
import { ROUTES } from '../config/routes.config';

const NotFoundPage: Component = () => {
  return (
    <div class="not-found-page">
      <h1>404</h1>
      <p>Page not found</p>
      <A href={ROUTES.DASHBOARD}>Go to Dashboard</A>
    </div>
  );
};

export default NotFoundPage;
