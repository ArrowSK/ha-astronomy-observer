#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WEB = ROOT / "astronomy_observer/web/index.html"
text = WEB.read_text(encoding="utf-8")


def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f"marker changed: {label}")
    text = text.replace(old, new, 1)


replace_once(
    '''    .target-thumb-wrap { width: 54px; height: 54px; position: relative; display: grid; place-items: center; overflow: hidden; border: 1px solid var(--line); border-radius: 9px; background: linear-gradient(145deg, #172238, #0d1422); }
    .target-thumb { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; display: block; }
    .target-thumb-fallback { color: #7183a1; font-size: 25px; line-height: 1; }
    .target-credit { position: absolute; right: 2px; bottom: 2px; width: 23px; height: 23px; display: grid; place-items: center; border-radius: 50%; background: rgba(9,13,22,.84); color: #d8e4f8; border: 1px solid rgba(255,255,255,.28); text-decoration: none; font-size: 12px; font-weight: 700; backdrop-filter: blur(5px); }
    .target-credit:hover { background: #1a2840; color: var(--accent); }''',
    '''    .target-media { width: 54px; display: flex; flex-direction: column; align-items: center; gap: 6px; }
    .target-thumb-wrap { width: 54px; height: 54px; position: relative; display: grid; place-items: center; overflow: hidden; border: 1px solid var(--line); border-radius: 9px; background: linear-gradient(145deg, #172238, #0d1422); }
    .target-thumb-button { padding: 0; cursor: zoom-in; touch-action: manipulation; }
    .target-thumb-button:hover { border-color: #4d607f; }
    .target-thumb { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; display: block; }
    .target-thumb-fallback { color: #7183a1; font-size: 25px; line-height: 1; }
    .target-credit { width: 26px; height: 26px; display: grid; place-items: center; border-radius: 50%; background: rgba(9,13,22,.84); color: #d8e4f8; border: 1px solid rgba(255,255,255,.28); text-decoration: none; font-size: 12px; font-weight: 700; }
    .target-credit:hover { background: #1a2840; color: var(--accent); }
    body.image-lightbox-open { overflow: hidden; }
    .image-lightbox[hidden] { display: none; }
    .image-lightbox { position: fixed; z-index: 100; inset: 0; display: flex; align-items: center; justify-content: center; padding: 24px; background: rgba(3,6,12,.94); }
    .image-lightbox-panel { position: relative; width: min(920px, 94vw); max-height: 92vh; display: flex; flex-direction: column; align-items: center; gap: 10px; }
    .image-lightbox-image { max-width: 100%; max-height: calc(92vh - 68px); object-fit: contain; border-radius: 12px; border: 1px solid var(--line); background: #070b13; }
    .image-lightbox-caption { color: #c7d3e7; font-size: 14px; text-align: center; }
    .image-lightbox-close { position: absolute; z-index: 2; top: -12px; right: -12px; width: 48px; height: 48px; padding: 0; display: grid; place-items: center; border-radius: 50%; background: #152138; border-color: #526480; font-size: 28px; line-height: 1; }
    .image-lightbox-close:hover { background: #1b2b47; }''',
    "target image CSS",
)

replace_once(
    '''      .target { grid-template-columns: 24px 48px minmax(0, 1fr) 42px; gap: 8px; }
      .target-thumb-wrap { width: 46px; height: 46px; border-radius: 8px; }
      .target-credit { width: 21px; height: 21px; font-size: 11px; }
      .target-score { font-size: 17px; }''',
    '''      .target { grid-template-columns: 24px 48px minmax(0, 1fr) 42px; gap: 8px; }
      .target-media { width: 46px; }
      .target-thumb-wrap { width: 46px; height: 46px; border-radius: 8px; }
      .target-credit { width: 24px; height: 24px; font-size: 11px; }
      .image-lightbox { padding: 14px; }
      .image-lightbox-panel { width: 100%; }
      .image-lightbox-close { top: 4px; right: 4px; }
      .image-lightbox-image { max-height: calc(100vh - 56px - env(safe-area-inset-top) - env(safe-area-inset-bottom)); }
      .target-score { font-size: 17px; }''',
    "mobile target image CSS",
)

replace_once(
    '''  function targetThumbnail(r) {
    const key = targetImageKey(r);
    const image = key ? `<img class="target-thumb" src="object-images/${key}.webp" alt="${esc(r.name)}" loading="lazy" decoding="async" onerror="this.remove()">` : '';
    const credit = key ? `<a class="target-credit" href="object-images/credits.html#${key}" title="Image credit and licence" aria-label="Image credit and licence for ${esc(r.name)}">i</a>` : '';
    return `<div class="target-thumb-wrap"><span class="target-thumb-fallback" aria-hidden="true">✦</span>${image}${credit}</div>`;
  }''',
    '''  function targetThumbnail(r) {
    const key = targetImageKey(r);
    if (!key) return `<div class="target-media"><div class="target-thumb-wrap"><span class="target-thumb-fallback" aria-hidden="true">✦</span></div></div>`;
    const imagePath = `object-images/${key}.webp`;
    const image = `<img class="target-thumb" src="${imagePath}" alt="${esc(r.name)}" loading="lazy" decoding="async" onerror="this.remove()">`;
    const credit = `<a class="target-credit" href="object-images/credits.html#${key}" title="Image credit and licence" aria-label="Image credit and licence for ${esc(r.name)}">i</a>`;
    return `<div class="target-media"><button class="target-thumb-wrap target-thumb-button" type="button" data-target-image="${imagePath}" data-target-name="${esc(r.name)}" aria-label="Expand image of ${esc(r.name)}"><span class="target-thumb-fallback" aria-hidden="true">✦</span>${image}</button>${credit}</div>`;
  }''',
    "target thumbnail renderer",
)

replace_once(
    '''  </div>
</main>
<nav id="bottom-nav" class="bottom-nav" aria-label="Astronomy Observer sections">''',
    '''  </div>
</main>
<div id="image-lightbox" class="image-lightbox" role="dialog" aria-modal="true" aria-labelledby="image-lightbox-caption" hidden>
  <div class="image-lightbox-panel">
    <button id="image-lightbox-close" class="image-lightbox-close" type="button" aria-label="Close expanded image" title="Close">×</button>
    <img id="image-lightbox-image" class="image-lightbox-image" alt="">
    <div id="image-lightbox-caption" class="image-lightbox-caption"></div>
  </div>
</div>
<nav id="bottom-nav" class="bottom-nav" aria-label="Astronomy Observer sections">''',
    "lightbox markup",
)

replace_once(
    '''document.addEventListener('keydown', event => {
  if (event.key === 'Escape') closeAppMenu();
});''',
    '''let lastTargetImageTrigger = null;

function openTargetImage(button) {
  const src = button.dataset.targetImage;
  if (!src) return;
  lastTargetImageTrigger = button;
  $('image-lightbox-image').src = src;
  $('image-lightbox-image').alt = button.dataset.targetName || 'Astronomy target';
  $('image-lightbox-caption').textContent = button.dataset.targetName || '';
  $('image-lightbox').hidden = false;
  document.body.classList.add('image-lightbox-open');
  $('image-lightbox-close').focus();
}

function closeTargetImage() {
  if ($('image-lightbox').hidden) return;
  $('image-lightbox').hidden = true;
  $('image-lightbox-image').removeAttribute('src');
  $('image-lightbox-image').alt = '';
  $('image-lightbox-caption').textContent = '';
  document.body.classList.remove('image-lightbox-open');
  if (lastTargetImageTrigger?.isConnected) lastTargetImageTrigger.focus();
  lastTargetImageTrigger = null;
}

$('target-list').addEventListener('click', event => {
  const button = event.target.closest('.target-thumb-button');
  if (button) openTargetImage(button);
});
$('image-lightbox-close').addEventListener('click', closeTargetImage);
$('image-lightbox').addEventListener('click', event => {
  if (event.target === $('image-lightbox')) closeTargetImage();
});

document.addEventListener('keydown', event => {
  if (event.key !== 'Escape') return;
  if (!$('image-lightbox').hidden) {
    closeTargetImage();
    return;
  }
  closeAppMenu();
});''',
    "lightbox behaviour",
)

WEB.write_text(text, encoding="utf-8")

validator = ROOT / "tests/validate_object_images.py"
v = validator.read_text(encoding="utf-8")
old_validator = '''    for marker in ["Wikimedia Commons", "not relicensed", "CC BY", "Public domain"]:
        require(marker in credits + notice + MANIFEST.read_text(encoding="utf-8"), f"thumbnail attribution marker missing: {marker}")

    print(f"Object thumbnail validation passed ({len(items)} licensed thumbnails)")'''
new_validator = '''    for marker in ["Wikimedia Commons", "not relicensed", "CC BY", "Public domain"]:
        require(marker in credits + notice + MANIFEST.read_text(encoding="utf-8"), f"thumbnail attribution marker missing: {marker}")

    ui = (ROOT / "astronomy_observer" / "web" / "index.html").read_text(encoding="utf-8")
    for ui_marker in [
        'class="target-thumb-wrap target-thumb-button"',
        'class="target-credit"',
        'id="image-lightbox"',
        'id="image-lightbox-close"',
        "function openTargetImage(button)",
        "function closeTargetImage()",
    ]:
        require(ui_marker in ui, f"target image lightbox marker missing: {ui_marker}")
    require('position: absolute; right: 2px; bottom: 2px' not in ui, "image credit must not overlay the thumbnail")

    print(f"Object thumbnail validation passed ({len(items)} licensed thumbnails)")'''
if old_validator not in v:
    raise SystemExit("marker changed: object image validator")
validator.write_text(v.replace(old_validator, new_validator, 1), encoding="utf-8")

readme = ROOT / "README.md"
r = readme.read_text(encoding="utf-8")
old = "Each accepted thumbnail keeps its own source, creator, licence and attribution in the bundled manifest and credits page. The credits page includes a clear return control in embedded views, while external source and licence links open separately. If a target has no accepted image, the interface simply falls back to the normal astronomy marker."
new = "Each accepted thumbnail keeps its own source, creator, licence and attribution in the bundled manifest and credits page. Tap or click a thumbnail to expand it for a proper look — especially useful on a phone — and close the full-screen view with the × button, the backdrop or Escape. The separate **i** credit control sits below the thumbnail so it does not compete with the image tap target. The credits page includes a clear return control in embedded views, while external source and licence links open separately. If a target has no accepted image, the interface simply falls back to the normal astronomy marker."
if old not in r:
    raise SystemExit("marker changed: README thumbnail paragraph")
readme.write_text(r.replace(old, new, 1), encoding="utf-8")

docs = ROOT / "astronomy_observer/DOCS.md"
d = docs.read_text(encoding="utf-8")
anchor = "## Top targets"
if anchor in d and "Tap or click a target thumbnail to expand it" not in d:
    d = d.replace(anchor, anchor + "\n\nTap or click a target thumbnail to expand it into a large, closable view. The separate **i** button below the thumbnail opens that image’s credit and licence information without getting in the way of the image tap target.", 1)
    docs.write_text(d, encoding="utf-8")

android_doc = ROOT / "docs/ANDROID.md"
a = android_doc.read_text(encoding="utf-8")
anchor = "The Android edition uses the same observing and ranking logic as the other editions."
if anchor in a and "tap an image to expand it" not in a:
    a = a.replace(anchor, anchor + " Target thumbnails use the same local image bundle: tap an image to expand it, use the close control to return, and use the separate **i** button below the thumbnail for credits and licence details.", 1)
    android_doc.write_text(a, encoding="utf-8")

print("Applied shared target image expansion and separated credit control")
