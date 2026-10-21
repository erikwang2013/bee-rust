/* Bee Rust site — shell interactions, dynamic sections, three.js boot */
(function () {
  'use strict';
  var i18n = window.BeeI18n;
  var data = window.BeeData || { pets: [], milestones: [] };
  var reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  var petBase = 'assets/img/pet-expressions/';
  var petPath = function (name) { return petBase + name + '.svg'; };
  var t = function (zh, en) { return i18n ? i18n.t(zh, en) : zh; };
  var lang = function () { return i18n ? i18n.lang : 'zh'; };

  /* Reveal on scroll */
  var revealObserver = null;
  function observeReveals(root) {
    var scope = root || document;
    var nodes = scope.querySelectorAll('[data-reveal]:not(.is-visible)');
    if (!('IntersectionObserver' in window)) {
      for (var i = 0; i < nodes.length; i++) nodes[i].classList.add('is-visible');
      return;
    }
    if (!revealObserver) {
      revealObserver = new IntersectionObserver(function (entries) {
        entries.forEach(function (entry) {
          if (entry.isIntersecting) {
            entry.target.classList.add('is-visible');
            revealObserver.unobserve(entry.target);
          }
        });
      }, { rootMargin: '0px 0px -8% 0px', threshold: 0.08 });
    }
    for (var j = 0; j < nodes.length; j++) revealObserver.observe(nodes[j]);
  }

  /* Dynamic pet gallery */
  function renderPets() {
    var host = document.getElementById('pet-grid');
    if (!host) return;
    var l = lang();
    host.innerHTML = data.pets.map(function (p) {
      var d = p[l] || p.zh;
      return '<figure class="pet-card" data-reveal>' +
        '<div class="pet-card-img"><img src="' + petPath(p.file) + '" alt="Rusty · ' + d.name + '" width="512" height="512" loading="lazy" decoding="async"></div>' +
        '<figcaption><strong>' + d.name + '</strong><span>' + d.scene + '</span></figcaption>' +
        '</figure>';
    }).join('');
    observeReveals(host);
  }

  /* Dynamic feature-evolution timeline */
  function renderMilestones() {
    var host = document.getElementById('milestones');
    if (!host) return;
    var l = lang();
    host.innerHTML = data.milestones.map(function (m) {
      var d = m[l] || m.zh;
      return '<article class="milestone" data-reveal>' +
        '<span class="milestone-dot" aria-hidden="true"></span>' +
        '<img class="milestone-pet" src="' + petPath(m.pet) + '" alt="" width="58" height="58" loading="lazy" decoding="async">' +
        '<div class="milestone-ver">' + m.version + ' <time>' + m.date + '</time></div>' +
        '<h4>' + d.title + '</h4><p>' + d.desc + '</p>' +
        '</article>';
    }).join('');
    observeReveals(host);
  }

  function setMeta(selector, value) {
    var el = document.querySelector(selector);
    if (el) el.setAttribute('content', value);
  }
  function applyMeta(title, description, locale) {
    document.title = title;
    setMeta('meta[name="description"]', description);
    setMeta('meta[property="og:title"]', title);
    setMeta('meta[property="og:description"]', description);
    setMeta('meta[property="og:locale"]', locale);
    setMeta('meta[name="twitter:title"]', title);
    setMeta('meta[name="twitter:description"]', description);
  }
  function syncPageMeta() {
    if (document.body.dataset.page !== 'home') return;
    var l = lang();
    applyMeta(
      l === 'en' ? 'Bee Rust · A production-grade Rust web framework' : 'Bee Rust · 生产级 Rust Web 框架',
      l === 'en'
        ? 'Bee Rust is a production-grade Rust web framework: Beego philosophy with traits, macros, and the type system. MVC, ORM, filter chains, CLI, and storage engines.'
        : 'Bee Rust 是生产级 Rust Web 框架：Beego 哲学，用 trait、macro 与类型系统表达。内置 MVC、ORM、过滤器链、CLI 与存储引擎。',
      l === 'en' ? 'en_US' : 'zh_CN'
    );
  }

  /* Header state + back-to-top visibility */
  var header = document.getElementById('site-header');
  var topBtn = document.getElementById('to-top');
  var ticking = false;
  function onScroll() {
    if (ticking) return;
    ticking = true;
    window.requestAnimationFrame(function () {
      var y = window.scrollY || document.documentElement.scrollTop;
      if (header) header.classList.toggle('is-scrolled', y > 24);
      if (topBtn) topBtn.classList.toggle('is-visible', y > 620);
      ticking = false;
    });
  }
  window.addEventListener('scroll', onScroll, { passive: true });
  onScroll();

  /* Mobile navigation */
  var menuBtn = document.querySelector('.menu-btn');
  var nav = document.getElementById('site-nav');
  function closeNav() {
    document.body.classList.remove('nav-open');
    if (menuBtn) menuBtn.setAttribute('aria-expanded', 'false');
  }
  if (menuBtn) {
    menuBtn.addEventListener('click', function () {
      var open = document.body.classList.toggle('nav-open');
      menuBtn.setAttribute('aria-expanded', open ? 'true' : 'false');
    });
  }
  if (nav) nav.addEventListener('click', function (e) { if (e.target.closest('a')) closeNav(); });
  document.addEventListener('keydown', function (e) { if (e.key === 'Escape') closeNav(); });
  var wide = window.matchMedia('(min-width: 981px)');
  if (wide.addEventListener) wide.addEventListener('change', function (e) { if (e.matches) closeNav(); });

  /* Active section link */
  var sections = document.querySelectorAll('main section[id]');
  var navLinks = document.querySelectorAll('[data-nav]');
  function setActive(id) {
    for (var i = 0; i < navLinks.length; i++) {
      var href = navLinks[i].getAttribute('href') || '';
      navLinks[i].classList.toggle('is-active', href === '#' + id);
    }
  }
  if ('IntersectionObserver' in window && sections.length && navLinks.length) {
    var sectionObserver = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) setActive(entry.target.id);
      });
    }, { rootMargin: '-42% 0px -52% 0px', threshold: 0 });
    for (var s = 0; s < sections.length; s++) sectionObserver.observe(sections[s]);
  }

  /* Copy buttons */
  document.addEventListener('click', function (e) {
    var btn = e.target.closest && e.target.closest('[data-copy]');
    if (!btn) return;
    var block = btn.closest('.code-block');
    var pre = block && block.querySelector('pre');
    if (!pre) return;
    var text = pre.textContent;
    var done = function () {
      btn.textContent = t('已复制', 'Copied');
      btn.classList.add('is-copied');
      window.setTimeout(function () {
        btn.textContent = lang() === 'en' ? (btn.getAttribute('data-en') || 'Copy') : '复制';
        btn.classList.remove('is-copied');
      }, 1500);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done).catch(function () { fallbackCopy(text, done); });
    } else {
      fallbackCopy(text, done);
    }
  });
  function fallbackCopy(text, done) {
    var area = document.createElement('textarea');
    area.value = text;
    area.setAttribute('readonly', '');
    area.style.position = 'fixed';
    area.style.opacity = '0';
    document.body.appendChild(area);
    area.select();
    try { document.execCommand('copy'); done(); } catch (e) { /* no clipboard */ }
    document.body.removeChild(area);
  }

  /* Donation QR lightbox */
  var qrModal = document.getElementById('qr-modal');
  var qrModalImg = document.getElementById('qr-modal-img');
  var qrModalTitle = document.getElementById('qr-modal-title');
  var qrTrigger = null;
  function qrName(trigger, suffixZh, suffixEn) {
    var zh = trigger.getAttribute('data-qr-name-zh') || '';
    var en = trigger.getAttribute('data-qr-name-en') || zh;
    return t(zh + suffixZh, en + suffixEn);
  }
  function syncQrText() {
    if (!qrTrigger) return;
    if (qrModalImg) qrModalImg.alt = qrName(qrTrigger, '打赏二维码', ' donation QR code');
    if (qrModalTitle) qrModalTitle.textContent = qrName(qrTrigger, '', '');
  }
  document.addEventListener('click', function (e) {
    var trigger = e.target.closest && e.target.closest('[data-qr]');
    if (!trigger) return;
    var src = trigger.getAttribute('data-qr');
    if (!src) return;
    if (!qrModal || typeof qrModal.showModal !== 'function') {
      window.open(src, '_blank', 'noopener');
      return;
    }
    qrTrigger = trigger;
    if (qrModalImg) qrModalImg.src = src;
    syncQrText();
    qrModal.showModal();
  });
  if (qrModal) {
    qrModal.addEventListener('click', function (e) { if (e.target === qrModal) qrModal.close(); });
    qrModal.addEventListener('close', function () { qrTrigger = null; });
    var qrClose = qrModal.querySelector('[data-qr-close]');
    if (qrClose) qrClose.addEventListener('click', function () { qrModal.close(); });
  }
  /* Warm QR decodes so the lightbox opens without a decode hitch */
  function warmQr() {
    var triggers = document.querySelectorAll('[data-qr]');
    for (var i = 0; i < triggers.length; i++) {
      var img = new Image();
      img.src = triggers[i].getAttribute('data-qr');
      if (img.decode) img.decode().catch(function () {});
    }
  }
  if ('requestIdleCallback' in window) window.requestIdleCallback(warmQr, { timeout: 2800 });
  else window.setTimeout(warmQr, 1800);

  /* Back to top — Rusty says hi */
  var topPet = document.getElementById('to-top-pet');
  var danceTimer = null;
  if (topBtn && topPet) {
    var preloadPets = function () {
      ['joy', 'knocking'].forEach(function (name) {
        var img = new Image();
        img.src = petPath(name);
      });
    };
    if ('requestIdleCallback' in window) window.requestIdleCallback(preloadPets, { timeout: 2800 });
    else window.setTimeout(preloadPets, 2000);
    topBtn.addEventListener('mouseenter', function () {
      if (!reduced) topPet.src = petPath('joy');
    });
    topBtn.addEventListener('mouseleave', function () {
      if (!topBtn.classList.contains('is-dancing')) topPet.src = petPath('cute');
    });
    topBtn.addEventListener('click', function () {
      topPet.src = petPath('knocking');
      topBtn.classList.add('is-dancing');
      window.clearTimeout(danceTimer);
      danceTimer = window.setTimeout(function () {
        topBtn.classList.remove('is-dancing');
        topPet.src = petPath('cute');
      }, 900);
      window.scrollTo({ top: 0, behavior: reduced ? 'auto' : 'smooth' });
    });
  }

  /* Three.js hero — load after the content has painted */
  function bootThree() {
    var canvas = document.getElementById('hero-canvas');
    if (!canvas || reduced || !window.WebGLRenderingContext) return;
    var conn = navigator.connection;
    if (conn && (conn.saveData || /(^|-)2g$/.test(conn.effectiveType || ''))) return;
    var run = function () {
      import('./hero-scene.min.js').then(function (mod) {
        if (mod && typeof mod.initHero === 'function') mod.initHero();
      }).catch(function () { /* CSS honeycomb stays as the fallback */ });
    };
    if ('requestIdleCallback' in window) window.requestIdleCallback(run, { timeout: 2800 });
    else window.setTimeout(run, 1600);
  }
  if (document.readyState === 'complete') bootThree();
  else window.addEventListener('load', bootThree, { once: true });

  /* Year + language changes */
  var years = document.querySelectorAll('[data-year]');
  for (var y = 0; y < years.length; y++) years[y].textContent = String(new Date().getFullYear());
  document.addEventListener('beerust:lang', function () {
    renderPets();
    renderMilestones();
    syncPageMeta();
    syncQrText();
  });

  renderPets();
  renderMilestones();
  observeReveals(document);
  syncPageMeta();
  window.BeeSite = { observeReveals: observeReveals, applyMeta: applyMeta };
})();
