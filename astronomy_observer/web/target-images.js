(() => {
  if (window.__astronomyObserverTargetImages) return;
  window.__astronomyObserverTargetImages = true;

  const THUMB_PX = 256;
  const FULL_PX = 1280;
  const FAILURE_TTL_MS = 10 * 60 * 1000;
  const cache = new Map();
  const pending = new Map();
  const misses = new Map();
  let manifestPromise = null;
  let activeButton = null;
  let activeToken = 0;

  function remoteAllowed() {
    return typeof window.AstronomyAndroid === 'undefined';
  }

  function lookupTitle(target) {
    const name = String(target?.name || '').replace(/\s+/g, ' ').trim();
    const category = String(target?.category || '').toLowerCase();
    if (!name) return null;

    const messier = name.match(/(?:^|[^A-Za-z])M\s*0*(\d{1,3})(?=\b|\s|\()/i);
    if (messier) return `Messier ${Number(messier[1])}`;
    const ngc = name.match(/\bNGC\s*0*(\d+)\b/i);
    if (ngc) return `NGC ${Number(ngc[1])}`;
    const ic = name.match(/\bIC\s*0*(\d+)\b/i);
    if (ic) return `IC ${Number(ic[1])}`;

    const lower = name.toLowerCase();
    const planets = new Set(['mercury', 'venus', 'mars', 'jupiter', 'saturn', 'uranus', 'neptune']);
    if (category.includes('planet') && planets.has(lower)) return lower === 'mercury' ? 'Mercury (planet)' : name;
    if (lower === 'moon') return 'Moon';
    if (lower === 'milky way' || lower.includes('galactic centre') || lower.includes('galactic center')) return 'Milky Way';
    if (category.includes('meteor') || category.includes('comet')) return name;
    if (category.includes('satellite') && /\bISS\b/i.test(name)) return 'International Space Station';

    if (category.includes('deep sky')) {
      const parts = name.split(/\s+[—–]\s+/);
      const common = parts.length > 1 ? parts.slice(1).join(' — ').replace(/\s*\([^)]*\)\s*$/, '').trim() : '';
      return common || name;
    }
    return null;
  }

  function bundledKey(button) {
    const src = String(button?.dataset?.targetImage || '');
    return src.match(/(?:^|\/)object-images\/([a-z0-9-]+)\.webp(?:$|[?#])/i)?.[1] || null;
  }

  function cleanMetadata(value) {
    if (!value) return '';
    const node = document.createElement('div');
    node.innerHTML = String(value);
    return String(node.textContent || '').replace(/\s+/g, ' ').trim();
  }

  function allowedLicense(value) {
    const normal = String(value || '').trim().toLowerCase();
    return normal === 'public domain'
      || normal.startsWith('cc0')
      || normal.startsWith('cc by ')
      || normal.startsWith('cc by-sa ');
  }

  async function json(url) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 8000);
    try {
      const response = await fetch(url, {cache: 'force-cache', signal: controller.signal});
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      return await response.json();
    } finally {
      clearTimeout(timer);
    }
  }

  async function manifest() {
    if (manifestPromise) return manifestPromise;
    manifestPromise = fetch('object-images/manifest.json', {cache: 'force-cache'})
      .then(response => response.ok ? response.json() : null)
      .catch(() => null);
    return manifestPromise;
  }

  async function bundledSourceFile(key) {
    if (!key) return null;
    const data = await manifest();
    const item = Array.isArray(data?.items) ? data.items.find(value => value?.key === key) : null;
    return item?.source_file || null;
  }

  function commonsPage(filename) {
    return `https://commons.wikimedia.org/wiki/File:${encodeURIComponent(String(filename || '').replaceAll(' ', '_'))}`;
  }

  async function resolveCommons(filename, size) {
    if (!remoteAllowed() || !filename) return null;
    const key = `commons:${size}:${String(filename).toLowerCase()}`;
    if (cache.has(key)) return cache.get(key);
    if (pending.has(key)) return pending.get(key);
    if ((misses.get(key) || 0) > Date.now()) return null;

    const task = (async () => {
      const params = new URLSearchParams({
        action: 'query', format: 'json', formatversion: '2', prop: 'imageinfo',
        iiprop: 'url|extmetadata', iiurlwidth: String(size), titles: `File:${filename}`, origin: '*'
      });
      const payload = await json(`https://commons.wikimedia.org/w/api.php?${params}`);
      const page = payload?.query?.pages?.find(value => Array.isArray(value?.imageinfo) && value.imageinfo.length);
      const info = page?.imageinfo?.[0];
      if (!info) return null;

      const metadata = info.extmetadata || {};
      const license = cleanMetadata(metadata?.LicenseShortName?.value);
      if (!allowedLicense(license)) return null;
      const imageUrl = info.thumburl || info.url;
      if (!imageUrl) return null;
      const parsed = new URL(imageUrl);
      if (parsed.protocol !== 'https:' || parsed.hostname !== 'upload.wikimedia.org') return null;

      const creator = cleanMetadata(metadata?.Artist?.value) || cleanMetadata(metadata?.Credit?.value);
      const sourceUrl = String(info.descriptionurl || '').startsWith('https://commons.wikimedia.org/')
        ? info.descriptionurl
        : commonsPage(filename);
      const result = {
        filename,
        imageUrl,
        sourceUrl,
        license,
        creator,
        creditText: [creator, license].filter(Boolean).join(' · ')
      };
      cache.set(key, result);
      return result;
    })().catch(() => null).then(result => {
      if (!result) misses.set(key, Date.now() + FAILURE_TTL_MS);
      return result;
    }).finally(() => pending.delete(key));

    pending.set(key, task);
    return task;
  }

  async function resolveTitle(title, size) {
    if (!remoteAllowed() || !title) return null;
    const key = `title:${size}:${title.toLowerCase()}`;
    if (cache.has(key)) return cache.get(key);
    if (pending.has(key)) return pending.get(key);
    if ((misses.get(key) || 0) > Date.now()) return null;

    const task = (async () => {
      const params = new URLSearchParams({
        action: 'query', format: 'json', formatversion: '2', redirects: '1',
        prop: 'pageimages', piprop: 'name', pilicense: 'free', titles: title, origin: '*'
      });
      const payload = await json(`https://en.wikipedia.org/w/api.php?${params}`);
      const filename = payload?.query?.pages?.find(value => value?.pageimage)?.pageimage;
      if (!filename) return null;
      const result = await resolveCommons(filename, size);
      if (result) cache.set(key, result);
      return result;
    })().catch(() => null).then(result => {
      if (!result) misses.set(key, Date.now() + FAILURE_TTL_MS);
      return result;
    }).finally(() => pending.delete(key));

    pending.set(key, task);
    return task;
  }

  async function resolveTarget(target, button, size) {
    if (!remoteAllowed()) return null;
    if (button?.dataset?.aoSourceFile) return resolveCommons(button.dataset.aoSourceFile, size);
    const key = bundledKey(button);
    if (key) {
      const filename = await bundledSourceFile(key);
      if (filename) return resolveCommons(filename, size);
    }
    return resolveTitle(lookupTitle(target), size);
  }

  function preload(url) {
    return new Promise((resolve, reject) => {
      const image = new Image();
      image.onload = resolve;
      image.onerror = reject;
      image.src = url;
    });
  }

  function recommendations() {
    try {
      return Array.isArray(currentSnapshot?.recommendations) ? currentSnapshot.recommendations : [];
    } catch (_) {
      return [];
    }
  }

  function addCredit(media, target) {
    let credit = media.querySelector('.target-credit');
    if (credit) return credit;
    credit = document.createElement('a');
    credit.className = 'target-credit';
    credit.textContent = 'i';
    credit.hidden = true;
    credit.title = 'Image credit and licence';
    credit.setAttribute('aria-label', `Image credit and licence for ${target?.name || 'target'}`);
    media.appendChild(credit);
    return credit;
  }

  function prepareRows() {
    const list = document.getElementById('target-list');
    if (!list) return;
    const values = recommendations();
    const rows = [...list.querySelectorAll('.target')];

    rows.forEach((row, index) => {
      const target = values[index];
      if (!target) return;
      let button = row.querySelector('.target-thumb-button');
      if (button) {
        button.dataset.aoIndex = String(index);
        return;
      }
      if (!remoteAllowed() || !lookupTitle(target)) return;
      const media = row.querySelector('.target-media');
      const wrap = media?.querySelector('.target-thumb-wrap');
      if (!media || !wrap) return;

      button = document.createElement('button');
      button.type = 'button';
      button.className = `${wrap.className} target-thumb-button`;
      button.dataset.aoIndex = String(index);
      button.dataset.aoDynamic = '1';
      button.dataset.targetName = target.name || '';
      button.setAttribute('aria-label', `Open image of ${target.name || 'target'}`);
      while (wrap.firstChild) button.appendChild(wrap.firstChild);
      wrap.replaceWith(button);
      addCredit(media, target);
    });

    void hydrateMissing(values);
  }

  async function hydrateMissing(values) {
    const list = document.getElementById('target-list');
    if (!list || !remoteAllowed()) return;
    const buttons = [...list.querySelectorAll('.target-thumb-button[data-ao-dynamic="1"]')]
      .filter(button => !button.dataset.aoState);
    let cursor = 0;

    async function worker() {
      while (cursor < buttons.length) {
        const button = buttons[cursor++];
        const target = values[Number(button.dataset.aoIndex)];
        if (!target) continue;
        button.dataset.aoState = 'loading';
        button.classList.add('image-loading');
        try {
          const result = await resolveTarget(target, button, THUMB_PX);
          if (!result || !button.isConnected) {
            button.dataset.aoState = 'miss';
            continue;
          }
          await preload(result.imageUrl);
          if (!button.isConnected) continue;
          const image = document.createElement('img');
          image.className = 'target-thumb';
          image.src = result.imageUrl;
          image.alt = target.name || '';
          image.loading = 'lazy';
          image.decoding = 'async';
          button.appendChild(image);
          button.dataset.targetImage = result.imageUrl;
          button.dataset.targetCredit = result.sourceUrl;
          button.dataset.aoSourceFile = result.filename;
          button.dataset.aoRemoteMeta = result.creditText || result.license || 'Wikimedia Commons';
          button.dataset.aoState = 'loaded';
          const credit = addCredit(button.parentElement, target);
          credit.href = result.sourceUrl;
          credit.target = '_blank';
          credit.rel = 'external noreferrer';
          credit.hidden = false;
          credit.title = result.creditText || 'Image source and licence';
        } catch (_) {
          button.dataset.aoState = 'miss';
        } finally {
          if (button.isConnected) button.classList.remove('image-loading');
        }
      }
    }

    await Promise.all([worker(), worker()]);
  }

  function ensureMeta() {
    let meta = document.getElementById('image-lightbox-meta');
    if (meta) return meta;
    const caption = document.getElementById('image-lightbox-caption');
    if (!caption) return null;
    meta = document.createElement('div');
    meta.id = 'image-lightbox-meta';
    meta.className = 'image-lightbox-meta';
    meta.setAttribute('aria-live', 'polite');
    caption.insertAdjacentElement('afterend', meta);
    return meta;
  }

  function showEmptyLightbox(button) {
    const lightbox = document.getElementById('image-lightbox');
    const image = document.getElementById('image-lightbox-image');
    const caption = document.getElementById('image-lightbox-caption');
    const credit = document.getElementById('image-lightbox-credit');
    if (!lightbox || !image || !caption || !credit) return false;
    image.removeAttribute('src');
    image.alt = button.dataset.targetName || 'Astronomy target';
    caption.textContent = button.dataset.targetName || '';
    credit.hidden = true;
    lightbox.hidden = false;
    document.body.classList.add('image-lightbox-open');
    document.getElementById('image-lightbox-close')?.focus();
    return true;
  }

  function setLightboxCredit(result) {
    const credit = document.getElementById('image-lightbox-credit');
    if (!credit || !result?.sourceUrl) return;
    credit.href = result.sourceUrl;
    credit.target = '_blank';
    credit.rel = 'external noreferrer';
    credit.hidden = false;
    const span = credit.querySelector('span');
    if (span) span.textContent = result.license ? `${result.license} · image source` : 'Image credit & licence';
  }

  async function upgradeLightbox(button, token) {
    const target = recommendations()[Number(button.dataset.aoIndex)];
    if (!target || !remoteAllowed()) return;
    const meta = ensureMeta();
    if (meta) meta.textContent = button.dataset.aoRemoteMeta || 'Loading a clearer licence-verified preview…';
    try {
      const result = await resolveTarget(target, button, FULL_PX);
      if (!result || token !== activeToken || activeButton !== button || document.getElementById('image-lightbox')?.hidden) {
        if (meta && token === activeToken && activeButton === button) meta.textContent = button.dataset.aoRemoteMeta || '';
        return;
      }
      await preload(result.imageUrl);
      if (token !== activeToken || activeButton !== button || document.getElementById('image-lightbox')?.hidden) return;
      const image = document.getElementById('image-lightbox-image');
      if (image) image.src = result.imageUrl;
      if (meta) meta.textContent = result.creditText || result.license || 'Wikimedia Commons';
      setLightboxCredit(result);
    } catch (_) {
      if (meta && token === activeToken && activeButton === button) meta.textContent = button.dataset.aoRemoteMeta || '';
    }
  }

  const baseOpenTargetImage = openTargetImage;
  openTargetImage = function(button) {
    activeButton = button;
    const token = ++activeToken;
    const hasImage = Boolean(button.dataset.targetImage || button.querySelector('img.target-thumb')?.src);
    if (hasImage) baseOpenTargetImage(button);
    else showEmptyLightbox(button);
    const meta = ensureMeta();
    if (meta) meta.textContent = button.dataset.aoRemoteMeta || '';
    void upgradeLightbox(button, token);
  };

  const baseCloseTargetImage = closeTargetImage;
  closeTargetImage = function() {
    const focusTarget = activeButton;
    ++activeToken;
    activeButton = null;
    baseCloseTargetImage();
    const meta = ensureMeta();
    if (meta) meta.textContent = '';
    const credit = document.getElementById('image-lightbox-credit');
    if (credit) {
      credit.removeAttribute('target');
      credit.removeAttribute('rel');
      const span = credit.querySelector('span');
      if (span) span.textContent = 'Image credit & licence';
    }
    if (focusTarget?.isConnected) focusTarget.focus();
  };

  const style = document.createElement('style');
  style.textContent = `
    .target-thumb-button.image-loading::after { content:""; position:absolute; z-index:3; width:18px; height:18px; border:2px solid rgba(238,243,255,.28); border-top-color:var(--accent); border-radius:50%; animation:spin .8s linear infinite; }
    .image-lightbox-panel { width:min(1120px,100%); }
    .image-lightbox-image { width:auto; height:auto; max-width:100%; max-height:calc(100dvh - 176px - env(safe-area-inset-top) - env(safe-area-inset-bottom)); }
    .image-lightbox-meta { min-height:18px; max-width:min(900px,100%); color:var(--muted); font-size:12px; line-height:1.4; text-align:center; }
  `;
  document.head.appendChild(style);

  const targetList = document.getElementById('target-list');
  if (targetList) {
    let scheduled = false;
    const schedule = () => {
      if (scheduled) return;
      scheduled = true;
      queueMicrotask(() => {
        scheduled = false;
        prepareRows();
      });
    };
    new MutationObserver(schedule).observe(targetList, {childList: true, subtree: true});
    schedule();
  }
})();
