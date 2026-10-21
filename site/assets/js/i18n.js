/* Bee Rust site — tiny bilingual switcher (zh in HTML, en in data-en attributes) */
(function () {
  'use strict';
  var KEY = 'beerust-lang';
  var root = document.documentElement;
  var lang = 'zh';
  try {
    var q = new URLSearchParams(location.search).get('lang');
    var saved = localStorage.getItem(KEY);
    lang = (q === 'en' || q === 'zh') ? q : (saved === 'en' ? 'en' : 'zh');
  } catch (e) { /* private mode / file:// */ }

  var nodes = [];
  var links = [];
  var arias = [];
  var toggles = [];

  function collect() {
    nodes = document.querySelectorAll('[data-en]');
    links = document.querySelectorAll('[data-en-href]');
    arias = document.querySelectorAll('[data-en-aria-label]');
    toggles = document.querySelectorAll('[data-lang-toggle]');
    for (var i = 0; i < nodes.length; i++) {
      if (!nodes[i].hasAttribute('data-zh-html')) nodes[i].setAttribute('data-zh-html', nodes[i].innerHTML);
    }
    for (var j = 0; j < links.length; j++) {
      if (!links[j].hasAttribute('data-zh-href')) links[j].setAttribute('data-zh-href', links[j].getAttribute('href') || '');
    }
    for (var k = 0; k < arias.length; k++) {
      if (!arias[k].hasAttribute('data-zh-aria-label')) arias[k].setAttribute('data-zh-aria-label', arias[k].getAttribute('aria-label') || '');
    }
  }

  function renderToggles() {
    for (var m = 0; m < toggles.length; m++) {
      toggles[m].textContent = lang === 'en' ? '中文' : 'EN';
      toggles[m].setAttribute('aria-label', lang === 'en' ? '切换为中文' : 'Switch to English');
      toggles[m].setAttribute('aria-pressed', lang === 'en' ? 'true' : 'false');
    }
  }

  function apply(next) {
    if (!nodes.length) collect();
    lang = next === 'en' ? 'en' : 'zh';
    root.lang = lang === 'en' ? 'en' : 'zh-CN';
    root.dataset.lang = lang;

    for (var i = 0; i < nodes.length; i++) {
      var el = nodes[i];
      var html = lang === 'en' ? el.getAttribute('data-en') : el.getAttribute('data-zh-html');
      if (el.innerHTML !== html) el.innerHTML = html;
    }
    for (var j = 0; j < links.length; j++) {
      links[j].setAttribute('href', lang === 'en' ? links[j].getAttribute('data-en-href') : links[j].getAttribute('data-zh-href'));
    }
    for (var k = 0; k < arias.length; k++) {
      arias[k].setAttribute('aria-label', lang === 'en' ? arias[k].getAttribute('data-en-aria-label') : arias[k].getAttribute('data-zh-aria-label'));
    }
    renderToggles();
    try { localStorage.setItem(KEY, lang); } catch (e) {}
    document.dispatchEvent(new CustomEvent('beerust:lang', { detail: { lang: lang } }));
  }

  window.BeeI18n = {
    get lang() { return lang; },
    t: function (zh, en) { return lang === 'en' ? en : zh; },
    apply: apply,
    toggle: function () { apply(lang === 'en' ? 'zh' : 'en'); }
  };

  function init() {
    toggles = document.querySelectorAll('[data-lang-toggle]');
    for (var i = 0; i < toggles.length; i++) toggles[i].addEventListener('click', window.BeeI18n.toggle);
    collect();
    if (lang === 'en') {
      apply(lang);
      return;
    }
    /* Chinese is already in the HTML: cache it, skip the no-op DOM walk */
    root.lang = 'zh-CN';
    root.dataset.lang = 'zh';
    renderToggles();
    try { localStorage.setItem(KEY, 'zh'); } catch (e) {}
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
  else init();
})();
