<template>
  <div class="landing">

    <!-- STAR FIELD -->
    <canvas ref="starCanvas" class="star-canvas" aria-hidden="true"></canvas>

    <!-- NAV -->
    <nav class="nav">
      <div class="nav-inner">
        <a href="/" class="nav-logo">
          <img src="/icon_round.png" alt="hoshi" class="nav-icon" />
          <span class="nav-name">hoshi</span>
        </a>
        <div class="nav-links">
          <a href="/hoshi/guide/introduction" class="nav-link">Docs</a>
          <a href="/hoshi/extensions/getting-started" class="nav-link">Extensions</a>
          <a href="https://github.com/dot-fx/hoshi" target="_blank" class="nav-gh" aria-label="GitHub">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>
          </a>
        </div>
      </div>
    </nav>

    <!-- HERO -->
    <section class="hero">
      <div class="container hero-inner">
        <div class="hero-text">
          <div class="hero-badges">
            <span class="badge badge-accent">AGPL-3.0</span>
            <span class="badge badge-outline">Open Source</span>
            <span
                v-if="latestRelease?.tag_name"
                class="badge badge-version"
            >
    {{ latestRelease.tag_name }}
  </span>
          </div>

          <h1 class="hero-title">
            <span class="title-main">hoshi</span>
            <span class="title-jp">星</span>
          </h1>

          <p class="hero-tagline">
            Unified media tracker and reader. Anime, manga, and novels. Lightweight, all in one place.
          </p>

          <div class="hero-cta">
            <a href="/hoshi/guide/introduction" class="btn btn-primary">Get Started</a>
            <a
                :href="latestRelease?.html_url || 'https://github.com/dot-fx/hoshi/releases/latest'"
                target="_blank"
                class="btn btn-secondary"
            >
              Download
            </a>
          </div>
        </div>

        <div class="hero-visual">
          <div class="character-wrapper">
            <img src="/hoshi.webp" class="character-slot" alt="Hoshi">
          </div>
        </div>
      </div>
    </section>

    <!-- FEATURES -->
    <section class="features">
      <div class="container">
        <h2 class="section-title">What makes hoshi yours</h2>

        <div class="feat-grid">
          <div class="feat-card">
            <h3>Tracking</h3>
            <p>Import your lists from AniList, MyAnimeList, and Kitsu. Everything stays in sync, automatically.</p>
          </div>

          <div class="feat-card">
            <h3>Extensions</h3>
            <p>Expand your sources with custom extensions. Full control over where and how you watch or read.</p>
          </div>

          <div class="feat-card">
            <h3>Customization</h3>
            <p>Tailor every detail - interface, player, reading experience. Make it feel like yours.</p>
          </div>
        </div>
      </div>
    </section>

    <!-- RELEASES -->
    <section class="releases">
      <div class="container releases-inner">
        <div class="rel-text">
          <h2 class="section-title">Available Platforms</h2>
          <p class="rel-desc">hoshi is currently in early development. Follow the GitHub repository for updates.</p>
        </div>

        <div class="rel-grid">

          <a :href="getDownloadLink('.exe')" target="_blank" class="platform">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4H10.949M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-12.9-1.801"/></svg>
            <span>Windows</span>
          </a>

          <a :href="getDownloadLink('.dmg')" target="_blank" class="platform">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.8-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/></svg>
            <span>macOS</span>
          </a>

          <a :href="getDownloadLink('.AppImage')" target="_blank" class="platform">
            <svg width="24" height="24" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><title>Linux</title><path d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139zm.529 3.405h.013c.213 0 .396.062.584.198.19.135.33.332.438.533.105.259.158.459.166.724 0-.02.006-.04.006-.06v.105a.086.086 0 01-.004-.021l-.004-.024a1.807 1.807 0 01-.15.706.953.953 0 01-.213.335.71.71 0 00-.088-.042c-.104-.045-.198-.064-.284-.133a1.312 1.312 0 00-.22-.066c.05-.06.146-.133.183-.198.053-.128.082-.264.088-.402v-.02a1.21 1.21 0 00-.061-.4c-.045-.134-.101-.2-.183-.333-.084-.066-.167-.132-.267-.132h-.016c-.093 0-.176.03-.262.132a.8.8 0 00-.205.334 1.18 1.18 0 00-.09.4v.019c.002.089.008.179.02.267-.193-.067-.438-.135-.607-.202a1.635 1.635 0 01-.018-.2v-.02a1.772 1.772 0 01.15-.768c.082-.22.232-.406.43-.533a.985.985 0 01.594-.2zm-2.962.059h.036c.142 0 .27.048.399.135.146.129.264.288.344.465.09.199.14.4.153.667v.004c.007.134.006.2-.002.266v.08c-.03.007-.056.018-.083.024-.152.055-.274.135-.393.2.012-.09.013-.18.003-.267v-.015c-.012-.133-.04-.2-.082-.333a.613.613 0 00-.166-.267.248.248 0 00-.183-.064h-.021c-.071.006-.13.04-.186.132a.552.552 0 00-.12.27.944.944 0 00-.023.33v.015c.012.135.037.2.08.334.046.134.098.2.166.268.01.009.02.018.034.024-.07.057-.117.07-.176.136a.304.304 0 01-.131.068 2.62 2.62 0 01-.275-.402 1.772 1.772 0 01-.155-.667 1.759 1.759 0 01.08-.668 1.43 1.43 0 01.283-.535c.128-.133.26-.2.418-.2zm1.37 1.706c.332 0 .733.065 1.216.399.293.2.523.269 1.052.468h.003c.255.136.405.266.478.399v-.131a.571.571 0 01.016.47c-.123.31-.516.643-1.063.842v.002c-.268.135-.501.333-.775.465-.276.135-.588.292-1.012.267a1.139 1.139 0 01-.448-.067 3.566 3.566 0 01-.322-.198c-.195-.135-.363-.332-.612-.465v-.005h-.005c-.4-.246-.616-.512-.686-.71-.07-.268-.005-.47.193-.6.224-.135.38-.271.483-.336.104-.074.143-.102.176-.131h.002v-.003c.169-.202.436-.47.839-.601.139-.036.294-.065.466-.065zm2.8 2.142c.358 1.417 1.196 3.475 1.735 4.473.286.534.855 1.659 1.102 3.024.156-.005.33.018.513.064.646-1.671-.546-3.467-1.089-3.966-.22-.2-.232-.335-.123-.335.59.534 1.365 1.572 1.646 2.757.13.535.16 1.104.021 1.67.067.028.135.06.205.067 1.032.534 1.413.938 1.23 1.537v-.043c-.06-.003-.12 0-.18 0h-.016c.151-.467-.182-.825-1.065-1.224-.915-.4-1.646-.336-1.77.465-.008.043-.013.066-.018.135-.068.023-.139.053-.209.064-.43.268-.662.669-.793 1.187-.13.533-.17 1.156-.205 1.869v.003c-.02.334-.17.838-.319 1.35-1.5 1.072-3.58 1.538-5.348.334a2.645 2.645 0 00-.402-.533 1.45 1.45 0 00-.275-.333c.182 0 .338-.03.465-.067a.615.615 0 00.314-.334c.108-.267 0-.697-.345-1.163-.345-.467-.931-.995-1.788-1.521-.63-.4-.986-.87-1.15-1.396-.165-.534-.143-1.085-.015-1.645.245-1.07.873-2.11 1.274-2.763.107-.065.037.135-.408.974-.396.751-1.14 2.497-.122 3.854a8.123 8.123 0 01.647-2.876c.564-1.278 1.743-3.504 1.836-5.268.048.036.217.135.289.202.218.133.38.333.59.465.21.201.477.335.876.335.039.003.075.006.11.006.412 0 .73-.134.997-.268.29-.134.52-.334.74-.4h.005c.467-.135.835-.402 1.044-.7zm2.185 8.958c.037.6.343 1.245.882 1.377.588.134 1.434-.333 1.791-.765l.211-.01c.315-.007.577.01.847.268l.003.003c.208.199.305.53.391.876.085.4.154.78.409 1.066.486.527.645.906.636 1.14l.003-.007v.018l-.003-.012c-.015.262-.185.396-.498.595-.63.401-1.746.712-2.457 1.57-.618.737-1.37 1.14-2.036 1.191-.664.053-1.237-.2-1.574-.898l-.005-.003c-.21-.4-.12-1.025.056-1.69.176-.668.428-1.344.463-1.897.037-.714.076-1.335.195-1.814.12-.465.308-.797.641-.984l.045-.022zm-10.814.049h.01c.053 0 .105.005.157.014.376.055.706.333 1.023.752l.91 1.664.003.003c.243.533.754 1.064 1.189 1.637.434.598.77 1.131.729 1.57v.006c-.057.744-.48 1.148-1.125 1.294-.645.135-1.52.002-2.395-.464-.968-.536-2.118-.469-2.857-.602-.369-.066-.61-.2-.723-.4-.11-.2-.113-.602.123-1.23v-.004l.002-.003c.117-.334.03-.752-.027-1.118-.055-.401-.083-.71.043-.94.16-.334.396-.4.69-.533.294-.135.64-.202.915-.47h.002v-.002c.256-.268.445-.601.668-.838.19-.201.38-.336.663-.336zm7.159-9.074c-.435.201-.945.535-1.488.535-.542 0-.97-.267-1.28-.466-.154-.134-.28-.268-.373-.335-.164-.134-.144-.333-.074-.333.109.016.129.134.199.2.096.066.215.2.36.333.292.2.68.467 1.167.467.485 0 1.053-.267 1.398-.466.195-.135.445-.334.648-.467.156-.136.149-.267.279-.267.128.016.034.134-.147.332a8.097 8.097 0 01-.69.468zm-1.082-1.583V5.64c-.006-.02.013-.042.029-.05.074-.043.18-.027.26.004.063 0 .16.067.15.135-.006.049-.085.066-.135.066-.055 0-.092-.043-.141-.068-.052-.018-.146-.008-.163-.065zm-.551 0c-.02.058-.113.049-.166.066-.047.025-.086.068-.14.068-.05 0-.13-.02-.136-.068-.01-.066.088-.133.15-.133.08-.031.184-.047.259-.005.019.009.036.03.03.05v.02h.003z"/></svg>
            <span>Linux</span>
          </a>

          <a :href="getDownloadLink('arm64')" target="_blank" class="platform">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M17.523 15.3414C17.523 15.8252 17.1306 16.2176 16.6468 16.2176C16.1629 16.2176 15.7705 15.8252 15.7705 15.3414C15.7705 14.8576 16.1629 14.4652 16.6468 14.4652C17.1306 14.4652 17.523 14.8576 17.523 15.3414ZM7.4475 16.2176C6.96365 16.2176 6.57127 15.8252 6.57127 15.3414C6.57127 14.8576 6.96365 14.4652 7.4475 14.4652C7.93134 14.4652 8.32371 14.8576 8.32371 15.3414C8.32371 15.8252 7.93134 16.2176 7.4475 16.2176ZM18.064 11.2384L19.9882 7.90483C20.0631 7.77353 20.02 7.6048 19.8887 7.52994C19.7574 7.45507 19.5887 7.49818 19.5138 7.62948L17.558 11.0182C15.9189 10.2662 14.0326 9.82776 12.0468 9.82776C10.0611 9.82776 8.17482 10.2662 6.5357 11.0182L4.57991 7.62948C4.50504 7.49818 4.3363 7.45507 4.20501 7.52994C4.07371 7.6048 4.0306 7.77353 4.10546 7.90483L6.02967 11.2384C2.66311 13.0827 0.380859 16.5912 0.0531201 20.7303H24.0405C23.7128 16.5912 21.4305 13.0827 18.064 11.2384Z"/></svg>
            <span>Android</span>
          </a>

        </div>
      </div>
    </section>

    <!-- FOOTER -->
    <footer class="footer">
      <div class="container footer-inner">
        <div class="footer-brand">
          <span class="footer-logo">hoshi</span>
          <span class="footer-license">GNU AGPL v3.0 · Open Source</span>
        </div>
        <div class="footer-nav">
          <a href="/hoshi/guide/introduction">Docs</a>
          <a href="https://github.com/dot-fx/hoshi" target="_blank">GitHub</a>
        </div>
      </div>
    </footer>

  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'

const starCanvas = ref(null)
let animationId = null

const latestRelease = ref(null)

onMounted(async () => {
  try {
    const res = await fetch(
        'https://api.github.com/repos/dot-fx/hoshi/releases/latest'
    )
    if (res.ok) {
      latestRelease.value = await res.json()
    }
  } catch (err) {
    console.error('Failed to fetch release:', err)
  }
})

// Ensures that we always return a valid URL, even if the fetch hasn't completed or failed.
const getDownloadLink = (keyword) => {
  const fallbackUrl = latestRelease.value?.html_url || 'https://github.com/dot-fx/hoshi/releases/latest';

  if (!latestRelease.value || !latestRelease.value.assets) {
    return fallbackUrl;
  }

  const asset = latestRelease.value.assets.find(a =>
      a.name.toLowerCase().includes(keyword.toLowerCase())
  );

  return asset ? asset.browser_download_url : fallbackUrl;
}

onMounted(() => {
  const canvas = starCanvas.value
  if (!canvas) return

  const ctx = canvas.getContext('2d')

  const resize = () => {
    canvas.width = window.innerWidth
    canvas.height = document.documentElement.scrollHeight
  }

  const STAR_COUNT = 1000
  const stars = []

  const initStars = () => {
    stars.length = 0
    for (let i = 0; i < STAR_COUNT; i++) {
      stars.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        r: Math.random() * 1.2 + 0.2,
        baseAlpha: Math.random() * 0.35 + 0.08,
        alpha: 0,
        speed: Math.random() * 0.004 + 0.001,
        phase: Math.random() * Math.PI * 2,
      })
    }
  }

  const draw = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height)
    for (const s of stars) {
      ctx.beginPath()
      ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2)
      ctx.fillStyle = `rgba(255, 255, 255, ${s.baseAlpha})`
      ctx.fill()
    }
  }

  const onResize = () => {
    resize()
    initStars()
  }

  window.addEventListener('resize', onResize)
  resize()
  initStars()
  resize()
  initStars()
  draw()

  onBeforeUnmount(() => {
    cancelAnimationFrame(animationId)
    window.removeEventListener('resize', onResize)
  })
})
</script>

<style scoped>
/* Same CSS as provided, no changes required. */
.star-canvas {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  pointer-events: none;
  z-index: 0;
}

.landing {
  position: relative;
  background-color: #09090b !important;
  color: #f4f4f5 !important;
  font-family: system-ui, -apple-system, sans-serif !important;
  min-height: 100vh;
  width: 100%;
}

.landing a, .landing p, .landing h1, .landing h2, .landing h3, .landing span {
  text-decoration: none !important;
  margin: 0;
}

.container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 64px;
}
/* NAV */
.nav {
  position: fixed; top: 0; left: 0; width: 100%;
  height: 64px;
  background-color: rgba(9, 9, 11, 0.85);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid #27272a;
  z-index: 100;
  display: flex; align-items: center;
}
.nav-inner {
  width: 100%;
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 64px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.nav-logo {
  display: flex; align-items: center; gap: 10px;
  font-weight: 700; font-size: 1.1rem; color: #fff !important;
}
.nav-icon { width: 26px; height: 26px; border-radius: 50%; }

.nav-links { display: flex; gap: 16px; align-items: center; }
.nav-link {
  color: #a1a1aa !important;
  font-size: 0.85rem; font-weight: 500;
  padding: 6px 8px;
  transition: color 0.2s ease;
}
.nav-gh {
  color: #a1a1aa !important;
  display: flex; align-items: center;
  padding: 6px;
  transition: color 0.2s ease;
}
.nav-link:hover, .nav-gh:hover { color: #fff !important; }

/* HERO */
.hero {
  padding-top: 160px;
  padding-bottom: 100px;
  position: relative;
  z-index: 1;
}
.hero-inner {
  display: flex; align-items: center; justify-content: space-between; gap: 60px;
}
.hero-text {
  flex: 1.2; display: flex; flex-direction: column; gap: 24px;
}
.hero-badges { display: flex; gap: 8px; }
.badge {
  padding: 4px 10px; border-radius: 6px; font-size: 0.75rem; font-weight: 600;
}
.badge-accent { background-color: #a78bfa; color: #000 !important; }
.badge-outline { border: 1px solid #3f3f46; color: #d4d4d8 !important; }
.badge-version {
  background-color: #18181b;
  color: #a1a1aa !important;
  border: 1px solid #27272a;
}

.hero-title {
  display: flex; align-items: baseline; gap: 16px;
}
.title-main { font-size: 5rem; font-weight: 900; color: #ffffff !important; line-height: 0.9; }
.title-jp { font-size: 2.5rem; color: #a78bfa !important; font-weight: 400; opacity: 0.8; }

.hero-tagline {
  font-size: 1.25rem; color: #a1a1aa !important; line-height: 1.6; max-width: 480px;
}

.hero-cta { display: flex; gap: 12px; margin-top: 8px; }
.btn {
  padding: 10px 20px; border-radius: 8px; font-weight: 600; font-size: 0.95rem;
  transition: all 0.2s ease; cursor: pointer;
}
.btn-primary { background-color: #ffffff; color: #000000 !important; }
.btn-primary:hover { background-color: #f4f4f5; transform: translateY(-1px); }
.btn-secondary { background-color: #18181b; color: #ffffff !important; border: 1px solid #27272a; }
.btn-secondary:hover { background-color: #27272a; border-color: #3f3f46; }

/* VISUAL */
.hero-visual {
  flex: 0.8; display: flex; justify-content: flex-end;
}
.character-slot {
  width: 100%;
  max-width: 380px;
  aspect-ratio: 1 / 1;

  background: transparent;
  border: none;

  border-radius: 24px;
  position: relative;
  z-index: 1;
}

.character-wrapper {
  position: relative;
  display: flex;
  justify-content: flex-end;
}

/* strong visible glow first */
.character-wrapper::before {
  content: "";
  position: absolute;
  width: 420px;
  height: 700px;

  top: 48%;
  left: 51%;
  transform: translate(-50%, -50%);

  background: radial-gradient(
      circle,
      rgba(255, 220, 120, 0.45) 0%,
      rgba(255, 220, 120, 0.25) 30%,
      rgba(255, 220, 120, 0.1) 55%,
      transparent 75%
  );

  filter: blur(60px);
  z-index: 0;
  pointer-events: none;
}

.character-slot {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 380px;
  border-radius: 24px;

  /* IMPORTANT: remove old bg */
  background: transparent;
  border: none;
}

/* FEATURES */
.features {
  padding: 100px 0; background: transparent; border-top: 1px solid #18181b;
  position: relative; z-index: 1;
}
.section-title {
  font-size: 2.25rem; font-weight: 800; color: #ffffff !important; margin-bottom: 48px !important; text-align: center;
}
.feat-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; }
.feat-card {
  background-color: #0f0f11; padding: 32px; border-radius: 16px; border: 1px solid #18181b;
}
.feat-card h3 { font-size: 1.2rem; color: #ffffff !important; margin-bottom: 12px !important; font-weight: 700; }
.feat-card p { font-size: 0.9rem; color: #a1a1aa !important; line-height: 1.6; }

/* RELEASES */
.releases {
  padding: 100px 0; border-top: 1px solid #18181b;
  position: relative; z-index: 1;
}
.releases-inner {
  display: flex; flex-direction: column; align-items: center; text-align: center; gap: 48px;
}
.rel-text { max-width: 500px; }
.rel-desc { font-size: 1rem; color: #a1a1aa !important; line-height: 1.6; }

.rel-grid {
  display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; width: 100%;
}
.platform {
  background: transparent; border: 1px solid #18181b; border-radius: 12px;
  padding: 20px; display: flex; flex-direction: column; align-items: center; gap: 12px;
  color: #d4d4d8 !important; font-weight: 600; font-size: 0.9rem;
}

/* FOOTER */
.footer {
  padding: 48px 0; border-top: 1px solid #18181b;
  position: relative; z-index: 1;
}
.footer-inner {
  display: flex; justify-content: space-between; align-items: center;
}
.footer-brand { display: flex; flex-direction: column; gap: 4px; }
.footer-logo { font-weight: 800; font-size: 1.1rem; color: #ffffff !important; }
.footer-license { font-size: 0.8rem; color: #52525b !important; }
.footer-nav { display: flex; gap: 20px; }
.footer-nav a { color: #71717a !important; font-size: 0.85rem; font-weight: 500; transition: color 0.2s ease; }
.footer-nav a:hover { color: #ffffff !important; }

@media (max-width: 900px) {
  .container { padding: 0 24px; }
  .nav-inner { padding: 0 24px; }
  .hero-inner { flex-direction: column; text-align: center; gap: 48px; }
  .hero-text { align-items: center; }
  .hero-tagline { max-width: 100%; }
  .hero-title { flex-direction: column; align-items: center; gap: 8px; }
  .title-main { font-size: 4rem; }
  .feat-grid { grid-template-columns: 1fr; }
  .rel-grid { grid-template-columns: repeat(2, 1fr); }
  .footer-inner { flex-direction: column; gap: 32px; text-align: center; }
}

@media (max-width: 600px) {
  .nav-links { display: none; }
  .title-main { font-size: 3.5rem; }
  .rel-grid { grid-template-columns: 1fr; }
}
</style>