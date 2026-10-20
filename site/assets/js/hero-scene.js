/* Bee Rust site — three.js hero scene
   Source file. In a scratch dir: npm i three esbuild
   Then: npx esbuild assets/js/hero-scene.js --bundle --minify --format=esm \
     --outfile=assets/js/hero-scene.min.js
   (three.js is tree-shaken into the bundle; no CDN request at runtime.) */
import {
  AdditiveBlending,
  BufferAttribute,
  BufferGeometry,
  CircleGeometry,
  Color,
  DoubleSide,
  Group,
  InstancedMesh,
  MathUtils,
  Mesh,
  MeshBasicMaterial,
  Object3D,
  PerspectiveCamera,
  Points,
  PointsMaterial,
  RingGeometry,
  Scene,
  WebGLRenderer
} from 'three';

export function initHero() {
  const canvas = document.getElementById('hero-canvas');
  if (!canvas || !canvas.parentElement) return;
  const host = canvas.parentElement;
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;

  let renderer;
  try {
    renderer = new WebGLRenderer({ canvas, alpha: true, antialias: true, powerPreference: 'high-performance' });
  } catch (e) {
    return;
  }

  const isSmall = window.matchMedia('(max-width: 760px)').matches;
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, isSmall ? 1.5 : 1.8));
  renderer.setClearColor(0x000000, 0);

  const scene = new Scene();
  const camera = new PerspectiveCamera(52, 1, 0.1, 120);
  camera.position.set(0, 0, 20);

  const world = new Group();
  scene.add(world);

  /* Hex-ring tunnel */
  const ringGeo = new RingGeometry(0.82, 0.94, 6, 1, Math.PI / 6);
  const ringMat = new MeshBasicMaterial({
    color: 0xffffff,
    transparent: true,
    opacity: 0.58,
    side: DoubleSide,
    depthWrite: false,
    blending: AdditiveBlending
  });
  const ringCount = isSmall ? 64 : 118;
  const rings = new InstancedMesh(ringGeo, ringMat, ringCount);
  const dummy = new Object3D();
  const amber = new Color('#f5b301');
  const rust = new Color('#b7410e');
  const wing = new Color('#cfe8f5');
  for (let i = 0; i < ringCount; i++) {
    const angle = Math.random() * Math.PI * 2;
    const depth = Math.random();
    const radius = 6.5 + Math.random() * 16;
    const scale = (0.28 + Math.random() * 0.95) * (1 - depth * 0.35);
    dummy.position.set(
      Math.cos(angle) * radius * 1.28,
      Math.sin(angle) * radius * 0.62,
      2 - depth * 42
    );
    dummy.rotation.set(0, 0, Math.random() * Math.PI);
    dummy.scale.setScalar(scale);
    dummy.updateMatrix();
    rings.setMatrixAt(i, dummy.matrix);
    const color = Math.random() > 0.74
      ? wing.clone()
      : amber.clone().lerp(rust, Math.random() * 0.78);
    color.multiplyScalar(0.48 + Math.random() * 0.55);
    rings.setColorAt(i, color);
  }
  rings.instanceMatrix.needsUpdate = true;
  if (rings.instanceColor) rings.instanceColor.needsUpdate = true;
  world.add(rings);

  /* Faint filled cells */
  const fillGeo = new CircleGeometry(0.9, 6);
  const fillMat = new MeshBasicMaterial({
    color: 0xf5b301,
    transparent: true,
    opacity: 0.04,
    side: DoubleSide,
    depthWrite: false,
    blending: AdditiveBlending
  });
  const fillCount = isSmall ? 20 : 34;
  const fills = new InstancedMesh(fillGeo, fillMat, fillCount);
  for (let i = 0; i < fillCount; i++) {
    const angle = Math.random() * Math.PI * 2;
    const radius = 7 + Math.random() * 14;
    dummy.position.set(
      Math.cos(angle) * radius * 1.24,
      Math.sin(angle) * radius * 0.58,
      -2 - Math.random() * 38
    );
    dummy.rotation.set(0, 0, Math.random() * Math.PI);
    dummy.scale.setScalar(1.6 + Math.random() * 3.4);
    dummy.updateMatrix();
    fills.setMatrixAt(i, dummy.matrix);
  }
  fills.instanceMatrix.needsUpdate = true;
  world.add(fills);

  /* Hex halo behind the mascot */
  const halo = new Group();
  halo.position.set(window.innerWidth > 900 ? 6.4 : 0, 0, 0);
  world.add(halo);
  const haloGeo = new RingGeometry(1.5, 1.56, 6);
  const haloMat = new MeshBasicMaterial({
    color: 0xf5b301,
    transparent: true,
    opacity: 0.24,
    side: DoubleSide,
    depthWrite: false,
    blending: AdditiveBlending
  });
  for (let i = 0; i < 6; i++) {
    const a = (i / 6) * Math.PI * 2;
    const ring = new Mesh(haloGeo, haloMat);
    ring.position.set(Math.cos(a) * 4.2, Math.sin(a) * 4.2 * 0.72, -3);
    ring.rotation.z = a;
    ring.scale.setScalar(2.6);
    halo.add(ring);
  }
  const core = new Mesh(
    new CircleGeometry(4.2, 6),
    new MeshBasicMaterial({ color: 0xf5b301, transparent: true, opacity: 0.025, side: DoubleSide, depthWrite: false })
  );
  core.position.set(0, 0, -4.5);
  halo.add(core);

  /* Honey dust */
  const dustCount = isSmall ? 240 : 480;
  const dustPositions = new Float32Array(dustCount * 3);
  const dustSpeeds = new Float32Array(dustCount);
  for (let i = 0; i < dustCount; i++) {
    dustPositions[i * 3] = (Math.random() - 0.5) * 48;
    dustPositions[i * 3 + 1] = (Math.random() - 0.5) * 28;
    dustPositions[i * 3 + 2] = -2 - Math.random() * 46;
    dustSpeeds[i] = 0.008 + Math.random() * 0.022;
  }
  const dustGeo = new BufferGeometry();
  dustGeo.setAttribute('position', new BufferAttribute(dustPositions, 3));
  const dust = new Points(dustGeo, new PointsMaterial({
    color: 0xffc62e,
    size: isSmall ? 0.07 : 0.09,
    transparent: true,
    opacity: 0.62,
    depthWrite: false,
    blending: AdditiveBlending,
    sizeAttenuation: true
  }));
  world.add(dust);

  /* A few cool wing-blue specks for contrast */
  const coolCount = isSmall ? 70 : 130;
  const coolPositions = new Float32Array(coolCount * 3);
  for (let i = 0; i < coolCount; i++) {
    coolPositions[i * 3] = (Math.random() - 0.5) * 44;
    coolPositions[i * 3 + 1] = (Math.random() - 0.5) * 26;
    coolPositions[i * 3 + 2] = -4 - Math.random() * 38;
  }
  const coolGeo = new BufferGeometry();
  coolGeo.setAttribute('position', new BufferAttribute(coolPositions, 3));
  const coolDust = new Points(coolGeo, new PointsMaterial({
    color: 0xcfe8f5,
    size: isSmall ? 0.055 : 0.07,
    transparent: true,
    opacity: 0.32,
    depthWrite: false,
    blending: AdditiveBlending,
    sizeAttenuation: true
  }));
  world.add(coolDust);

  const pointer = { x: 0, y: 0, tx: 0, ty: 0 };
  let firstFrame = true;
  let running = false;

  function resize() {
    const w = host.clientWidth;
    const h = host.clientHeight;
    if (!w || !h) return;
    renderer.setSize(w, h, false);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }
  resize();
  if ('ResizeObserver' in window) new ResizeObserver(resize).observe(host);
  else window.addEventListener('resize', resize, { passive: true });

  window.addEventListener('pointermove', function (e) {
    pointer.tx = (e.clientX / window.innerWidth - 0.5) * 2;
    pointer.ty = (e.clientY / window.innerHeight - 0.5) * 2;
  }, { passive: true });

  function tick(now) {
    const time = now * 0.001;
    pointer.x = MathUtils.lerp(pointer.x, pointer.tx, 0.045);
    pointer.y = MathUtils.lerp(pointer.y, pointer.ty, 0.045);

    world.rotation.y = pointer.x * 0.08 + Math.sin(time * 0.07) * 0.035;
    world.rotation.x = -pointer.y * 0.055;
    world.rotation.z = Math.sin(time * 0.05) * 0.022;
    world.position.y = Math.sin(time * 0.18) * 0.22;
    halo.rotation.z = time * 0.075;
    halo.rotation.y = Math.sin(time * 0.12) * 0.16;

    const arr = dustGeo.attributes.position.array;
    for (let i = 0; i < dustCount; i++) {
      arr[i * 3 + 1] += dustSpeeds[i];
      if (arr[i * 3 + 1] > 14) arr[i * 3 + 1] = -14;
    }
    dustGeo.attributes.position.needsUpdate = true;

    const scroll = Math.min((window.scrollY || 0) / Math.max(host.clientHeight, 1), 1);
    camera.position.z = 20 - scroll * 3.2;
    camera.position.x = pointer.x * 0.6;
    camera.position.y = -pointer.y * 0.42;
    camera.lookAt(0, 0, 0);
    renderer.render(scene, camera);

    if (firstFrame) {
      firstFrame = false;
      host.classList.add('webgl-ready');
    }
  }

  function start() {
    if (running) return;
    running = true;
    renderer.setAnimationLoop(tick);
  }
  function stop() {
    if (!running) return;
    running = false;
    renderer.setAnimationLoop(null);
  }

  if ('IntersectionObserver' in window) {
    const visibilityObserver = new IntersectionObserver(function (entries) {
      const visible = entries[0] && entries[0].isIntersecting;
      if (visible && !document.hidden) start();
      else stop();
    }, { threshold: 0.01 });
    visibilityObserver.observe(host);
  } else {
    start();
  }
  document.addEventListener('visibilitychange', function () {
    if (document.hidden) { stop(); return; }
    const rect = host.getBoundingClientRect();
    if (rect.bottom > 0 && rect.top < window.innerHeight) start();
  });

  canvas.addEventListener('webglcontextlost', function (e) {
    e.preventDefault();
    stop();
    host.classList.remove('webgl-ready');
  });

  window.addEventListener('pagehide', function (e) {
    if (e.persisted) return;
    stop();
    ringGeo.dispose();
    fillGeo.dispose();
    haloGeo.dispose();
    dustGeo.dispose();
    coolGeo.dispose();
    ringMat.dispose();
    fillMat.dispose();
    haloMat.dispose();
    renderer.dispose();
  }, { once: true });
}
