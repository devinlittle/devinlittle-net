/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />
/// <reference types="@sveltejs/kit" />
/// <reference types="../.svelte-kit/ambient.d.ts" />

import { self } from '$app/service-worker';
import { version } from '$app/env';
import { immutable, assets } from '$app/manifest';
import { resolve } from '$app/paths';

const CACHE = `cache-${version}`;
const ASSETS = [
  ...immutable.map((asset) => resolve(asset.path)),
  ...assets.map((asset) => resolve(asset.path))
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => {
      return Promise.all(
        ASSETS.map(url => cache.add(url).catch(() => {
          console.warn(`[SW] Could not cache: ${url}`);
        }))
      );
    }).then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      await caches.keys().then((keys) => {
        return Promise.all(
          keys.filter(key => key !== CACHE).map(key => caches.delete(key))
        );
      }).then(() => self.clients.claim())
    })()
  );
});

self.addEventListener('push', (event) => {
  if (!event.data) return;

  event.waitUntil(
    (async () => {
      try {
        const data = event.data?.json();
        console.log('[SW] push received:', data);

        const title = data?.payload?.title ?? data?.title ?? 'Notification';
        const content = data?.payload?.content ?? data?.body ?? '';
        const sender_username = data?.payload?.sender_username ?? '';

        await self.registration.showNotification(`${title} - ${sender_username}`, {
          body: content,
          icon: '/favicon.png',
          badge: '/favicon.png',
          data: data?.payload ?? data
        });
      } catch (err) {
        console.error('[SW] Error processing push data:', err);

        await self.registration.showNotification('New Message', {
          body: 'You received a new notification.',
          icon: '/favicon.png'
        });
      }
    })()
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  event.waitUntil(self.clients.openWindow('/'));
});
