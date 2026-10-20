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

  function apply(next) {
    lang = next === 'en' ? 'en' : 'zh';
    root.lang = lang === 'en' ? 'en' : 'zh-CN';
    root.dataset.lang = lang;

    var nodes = document.querySelectorAll('[data-en]');
    for (var i = 0; i < nodes.length; i++) {
      var el = nodes[i];
      if (!el.hasAttribute('data-zh-html')) el.setAttribute('data-zh-html', el.innerHTML);
      el.innerHTML = lang === 'en' ? el.getAttribute('data-en') : el.getAttribute('data-zh-html');
    }
    var links = document.querySelectorAll('[data-en-href]');
    for (var j = 0; j < links.length; j++) {
      var link = links[j];
      if (!link.hasAttribute('data-zh-href')) link.setAttribute('data-zh-href', link.getAttribute('href') || '');
      link.setAttribute('href', lang === 'en' ? link.getAttribute('data-en-href') : link.getAttribute('data-zh-href'));
    }
    var arias = document.querySelectorAll('[data-en-aria-label]');
    for (var k = 0; k < arias.length; k++) {
      var a = arias[k];
      if (!a.hasAttribute('data-zh-aria-label')) a.setAttribute('data-zh-aria-label', a.getAttribute('aria-label') || '');
      a.setAttribute('aria-label', lang === 'en' ? a.getAttribute('data-en-aria-label') : a.getAttribute('data-zh-aria-label'));
    }
    var toggles = document.querySelectorAll('[data-lang-toggle]');
    for (var m = 0; m < toggles.length; m++) {
      toggles[m].textContent = lang === 'en' ? '中文' : 'EN';
      toggles[m].setAttribute('aria-label', lang === 'en' ? '切换为中文' : 'Switch to English');
      toggles[m].setAttribute('aria-pressed', lang === 'en' ? 'true' : 'false');
    }
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
    var toggles = document.querySelectorAll('[data-lang-toggle]');
    for (var i = 0; i < toggles.length; i++) toggles[i].addEventListener('click', window.BeeI18n.toggle);
    apply(lang);
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
  else init();
})();
