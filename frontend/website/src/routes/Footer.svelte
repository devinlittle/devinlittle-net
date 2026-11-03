<script lang="ts" context="module">
  declare const __GIT_HASH__: string;
</script>

<footer>
  <div class="footer-left">
    <span class="label">Running on commit</span>
    <a
      href="https://github.com/devinlittle/devinlittle-net/commit/{__GIT_HASH__}"
      target="_blank"
      rel="noopener noreferrer"
      class="hash"
    >
      {__GIT_HASH__}
    </a>
  </div>

  <div class="footer-right">
    <a
      href="https://github.com/devinlittle/devinlittle-net"
      class="github-link"
      aria-label="Source on GitHub"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path
          d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"
        />
      </svg>
    </a>
  </div>
</footer>

<style>
  footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.65rem 1rem;
    background: rgba(41, 11, 68, 0.7);
    backdrop-filter: blur(6px);
    border-top: 1px solid var(--color-border);
    font-family: var(--font-body);
    color: var(--color-text);
    z-index: 40;
  }

  .footer-left {
    display: flex;
    align-items: baseline;
    gap: 0.55rem;
  }

  .label {
    color: var(--color-subtle-text);
    font-size: 0.7rem;
    font-weight: bold;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .hash {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--color-theme-2);
    text-decoration: none;
    letter-spacing: 0.01em;
    transition: color 0.2s ease;
  }

  .hash:hover {
    color: var(--color-theme-1);
  }

  .footer-right {
    display: flex;
    align-items: center;
  }

  .github-link {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-subtle-text);
    opacity: 0.75;
    transition:
      opacity 0.2s ease,
      color 0.2s ease;
  }

  .github-link:hover {
    opacity: 1;
    color: var(--color-theme-1);
  }

  @media (max-width: 768px) {
    footer {
      padding: 0.5rem 0.75rem;
    }

    .label {
      display: none;
    }

    .hash {
      font-size: 0.72rem;
    }
  }
</style>
