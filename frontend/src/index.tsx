/* @refresh reload */
import { render } from 'solid-js/web';
import App from './App';
import { pwaService } from './services/pwa.service';

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? ' +
    'Or maybe the id attribute got misspelled?',
  );
}

// Регистрируем сервис-воркер после загрузки DOM
document.addEventListener('DOMContentLoaded', () => {
  // Регистрируем PWA только в production или если включен в dev режиме
  if (import.meta.env.PROD || import.meta.env.VITE_PWA_DEV === 'true') {
    pwaService.register().catch(console.error);
  }
});

render(() => <App />, root!);