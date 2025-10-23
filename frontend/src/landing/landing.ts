/**
 * Landing Page - Vanilla TypeScript
 * Simple landing page for link redirection
 */

/**
 * Create and render landing page
 */
function createLandingPage(): void {
  const app = document.getElementById('app');
  if (!app) return;

  app.innerHTML = `
    <div class="landing">
      <!-- Hero Section -->
      <header class="hero">
        <div class="container">
          <h1 class="hero-title">
            <span class="gradient-text">Links</span>
          </h1>
          <p class="hero-subtitle">
            Fast and simple URL shortener
          </p>
        </div>
      </header>

      <!-- Features Section -->
      <section class="features">
        <div class="container">
          <h2 class="section-title">Why choose Links?</h2>
          <div class="features-grid">
            <div class="feature-card">
              <div class="feature-icon">⚡</div>
              <h3 class="feature-title">Lightning Fast</h3>
              <p class="feature-text">
                Instant redirects with minimal overhead
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-icon">🔒</div>
              <h3 class="feature-title">Secure</h3>
              <p class="feature-text">
                All links are validated and safe to use
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-icon">📊</div>
              <h3 class="feature-title">Analytics</h3>
              <p class="feature-text">
                Track clicks and monitor link performance
              </p>
            </div>
          </div>

          <div class="cta-section">
            <a href="/admin" class="btn btn-primary">Go to Admin Panel</a>
          </div>
        </div>
      </section>

      <!-- Footer -->
      <footer class="footer">
        <div class="container">
          <p class="footer-text">
            Links - URL Shortener Service
          </p>
        </div>
      </footer>
    </div>
  `;
}

// Initialize landing page
createLandingPage();