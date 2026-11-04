/* Bee Rust site — client-side bilingual site search */
(function () {
  'use strict';
  var i18n = window.BeeI18n;
  var t = function (zh, en) { return i18n ? i18n.t(zh, en) : zh; };
  var lang = function () { return i18n ? i18n.lang : 'zh'; };
  var actions = document.querySelector('.header-actions');
  if (!actions) return;

  var page = document.body.dataset.page || '';
  var isHome = page === 'home';
  var isLog = page === 'changelog';
  var is404 = page === '404';
  var homePrefix = isHome ? '#' : (is404 ? '/#' : './#');
  var logPrefix = isLog ? '#' : (is404 ? '/changelog.html#' : 'changelog.html#');
  var shortcut = /Mac|iPhone|iPad/.test(navigator.platform || '') ? '⌘K' : 'Ctrl+K';

  function homeHref(id) { return homePrefix + id; }
  function logHref(version) { return logPrefix + 'v' + version.replace(/\./g, '-'); }

  var SECTIONS = [
    { id: 'top', zh: 'Bee Rust 首页', en: 'Bee Rust home',
      zhBody: '生产级 Rust Web 框架，把 Beego 的 MVC、命名空间与过滤器链带进 Rust。',
      enBody: 'A production-grade Rust web framework bringing Beego MVC, namespaces, and filter chains to Rust.',
      keys: 'home hero overview 首页 概览 开始' },
    { id: 'intro', zh: '项目介绍', en: 'Project intro',
      zhBody: '保留 Beego 的顺手，用 trait、macro 与类型系统重新表达，默认 full 特性开箱即用。',
      enBody: 'Beego ergonomics expressed with Rust traits, macros, and the type system; the default full feature works out of the box.',
      keys: 'project about beego 项目 介绍 定位' },
    { id: 'features', zh: '功能全景', en: 'Features',
      zhBody: '路由、ORM、安全、配置、日志、缓存、Session、模板与五类存储引擎。',
      enBody: 'Routing, ORM, security, config, logs, cache, sessions, templates, and five families of storage engines.',
      keys: 'capabilities ORM MySQL 分库分表 sharding Redis 安全 security 缓存 cache CLI 功能' },
    { id: 'architecture', zh: '架构', en: 'Architecture',
      zhBody: '分层架构：bee_rust 元 crate 重新导出可独立使用的 crate，feature gate 按需开启。',
      enBody: 'Layered architecture: the bee_rust meta crate re-exports independently usable crates behind feature gates.',
      keys: 'architecture crate feature flags axum tokio 架构 分层' },
    { id: 'rusty', zh: 'Rusty', en: 'Rusty',
      zhBody: '一只把依赖锁死的工程师蜜蜂，以及它的十二种表情。',
      enBody: 'The engineer bee who pins every dependency, in twelve expressions.',
      keys: 'rusty mascot pet 蜜蜂 表情 吉祥物' },
    { id: 'install', zh: '安装与快速开始', en: 'Install and quick start',
      zhBody: 'Rust 1.85+，cargo add bee_rust，或用 bee-rust new 创建项目。',
      enBody: 'Rust 1.85+, cargo add bee_rust, or scaffold a project with bee-rust new.',
      keys: 'install quick start cargo add rust 1.85 cli 安装 快速开始' },
    { id: 'usage', zh: '使用指南', en: 'Usage',
      zhBody: '从第一个路由到第一张表：控制器、路由、ORM 与完整示例。',
      enBody: 'From the first route to the first table: controllers, routes, ORM, and complete examples.',
      keys: 'usage route controller orm example 使用 路由 控制器 示例' },
    { id: 'faq', zh: '常见问题', en: 'FAQ',
      zhBody: '关于框架定位、Beego 差异、数据库支持、安装与生产可用性的常见问题。',
      enBody: 'FAQ on positioning, Beego differences, database support, installation, and production readiness.',
      keys: 'faq question database production 常见问题 数据库 生产' },
    { id: 'download', zh: '下载', en: 'Download',
      zhBody: 'crates.io、GitHub Release、docs.rs 与源码仓库。',
      enBody: 'crates.io, GitHub releases, docs.rs, and the source repository.',
      keys: 'download crates.io github release docs.rs 1.6.2 下载' },
    { id: 'support', zh: '支持项目', en: 'Support',
      zhBody: '用微信支付或支付宝支持 Bee Rust 与 Rusty。',
      enBody: 'Support Bee Rust and Rusty with WeChat Pay or Alipay.',
      keys: 'support donation wechat alipay 支持 打赏 微信 支付宝' }
  ];
  var QUICK_IDS = ['top', 'features', 'install', 'usage', 'architecture', 'faq', 'download'];

  var RESOURCES = [
    { zh: 'README', en: 'README', zhBody: '项目总览、安装、快速开始与功能说明。',
      enBody: 'Project overview, installation, quick start, and capabilities.',
      hrefZh: 'https://github.com/erikwang2013/bee-rust/blob/main/README.md',
      hrefEn: 'https://github.com/erikwang2013/bee-rust/blob/main/docs/README.en.md',
      keys: 'readme docs 文档 说明' },
    { zh: 'API 参考', en: 'API reference', zhBody: '每个 crate 与模块的完整 API 文档。',
      enBody: 'Complete API documentation for every crate and module.',
      hrefZh: 'https://github.com/erikwang2013/bee-rust/blob/main/docs/api.md',
      hrefEn: 'https://github.com/erikwang2013/bee-rust/blob/main/docs/api.en.md',
      keys: 'api reference docs 文档 参考' },
    { zh: 'docs.rs', en: 'docs.rs', zhBody: '由源码渲染的全部 crate 与 feature。',
      enBody: 'Every crate and feature rendered from source.',
      hrefZh: 'https://docs.rs/bee_rust', hrefEn: 'https://docs.rs/bee_rust',
      keys: 'docs.rs rustdoc api 文档' },
    { zh: 'GitHub 仓库', en: 'GitHub repository', zhBody: '源码、示例、issue 与完整提交历史。',
      enBody: 'Source, examples, issues, and the complete history.',
      hrefZh: 'https://github.com/erikwang2013/bee-rust',
      hrefEn: 'https://github.com/erikwang2013/bee-rust',
      keys: 'github source repository 源码 仓库' },
    { zh: 'crates.io', en: 'crates.io', zhBody: 'bee_rust 元 crate 与全部子 crate。',
      enBody: 'The bee_rust meta crate and every sub-crate.',
      hrefZh: 'https://crates.io/crates/bee_rust', hrefEn: 'https://crates.io/crates/bee_rust',
      keys: 'crates.io cargo package 发布 版本' }
  ];
  var index = null;
  var dialog = null;
  var input = null;
  var list = null;
  var empty = null;
  var count = null;
  var hint = null;
  var closeBtn = null;
  var titleEl = null;
  var trigger = null;
  var lastFocus = null;
  var results = [];
  var active = -1;

  function textOf(el, which) {
    if (!el) return '';
    var html = which === 'en'
      ? (el.getAttribute('data-en') || el.getAttribute('data-zh-html') || el.innerHTML)
      : (el.getAttribute('data-zh-html') || el.innerHTML);
    var box = document.createElement('div');
    box.innerHTML = html;
    return (box.textContent || '').replace(/\s+/g, ' ').trim();
  }

  function addCards(out, selector, section, type) {
    var cards = document.querySelectorAll(selector);
    for (var i = 0; i < cards.length; i++) {
      var card = cards[i];
      var heading = card.querySelector('h3, h4');
      if (!heading) continue;
      var body = card.querySelector('p');
      var tags = card.querySelector('.tags');
      out.push({ type: type, title: { zh: textOf(heading, 'zh'), en: textOf(heading, 'en') },
        body: { zh: textOf(body, 'zh'), en: textOf(body, 'en') }, href: homeHref(section),
        keys: tags ? textOf(tags, 'zh') + ' ' + textOf(tags, 'en') : '' });
    }
  }

  function buildIndex() {
    var out = [];
    var i;
    for (i = 0; i < SECTIONS.length; i++) {
      var s = SECTIONS[i];
      out.push({ type: 'section', quick: QUICK_IDS.indexOf(s.id) !== -1,
        title: { zh: s.zh, en: s.en }, body: { zh: s.zhBody, en: s.enBody },
        href: homeHref(s.id), keys: s.keys });
    }
    if (!isLog) out.push({ type: 'page', quick: true,
      title: { zh: '版本日志', en: 'Changelog' },
      body: { zh: '从 1.0.3 到 1.6.2 的每次迭代，按版本查看新增、修复与清理。',
        en: 'Every release from 1.0.3 to 1.6.2: additions, fixes, and cleanups.' },
      href: is404 ? '/changelog.html' : 'changelog.html',
      keys: 'changelog release notes 版本 日志 更新' });
    for (i = 0; i < RESOURCES.length; i++) {
      var r = RESOURCES[i];
      out.push({ type: 'resource', title: { zh: r.zh, en: r.en },
        body: { zh: r.zhBody, en: r.enBody }, hrefZh: r.hrefZh, hrefEn: r.hrefEn, keys: r.keys });
    }
    if (isHome) {
      addCards(out, '.goal-card', 'intro', 'feature');
      addCards(out, '.feature-card', 'features', 'feature');
      addCards(out, '.faq-card', 'faq', 'faq');
      addCards(out, '.download-card', 'download', 'resource');
      var data = window.BeeData;
      if (data && data.milestones) {
        for (i = 0; i < data.milestones.length; i++) {
          var m = data.milestones[i];
          var mz = m.zh || {};
          var me = m.en || mz;
          out.push({ type: 'milestone',
            title: { zh: 'v' + m.version + ' · ' + (mz.title || ''), en: 'v' + m.version + ' · ' + (me.title || '') },
            body: { zh: mz.desc || '', en: me.desc || '' }, href: homeHref('features'),
            keys: m.version + ' ' + m.date + ' 里程碑 milestone' });
        }
      }
    }
    var log = window.BeeChangelog || [];
    for (i = 0; i < log.length; i++) {
      var rel = log[i];
      var rz = rel.zh || {};
      var re = rel.en || rz;
      out.push({ type: 'release',
        title: { zh: 'v' + rel.version + ' · ' + (rz.title || ''), en: 'v' + rel.version + ' · ' + (re.title || '') },
        body: { zh: [rz.summary || ''].concat(rz.items || []).join(' '),
          en: [re.summary || ''].concat(re.items || []).join(' ') },
        href: logHref(rel.version), keys: rel.version + ' ' + rel.date + ' ' + (rel.tag || '') });
    }
    for (i = 0; i < out.length; i++) {
      var e = out[i];
      e._tz = String(e.title.zh || '').toLowerCase();
      e._te = String(e.title.en || '').toLowerCase();
      e._bz = String(e.body.zh || '').toLowerCase();
      e._be = String(e.body.en || '').toLowerCase();
      e._all = [e._tz, e._te, e._bz, e._be, String(e.keys || '').toLowerCase()].join(' ');
    }
    return out;
  }

  function getIndex() {
    if (!index) index = buildIndex();
    return index;
  }
  function hrefOf(e) {
    return lang() === 'en' && e.hrefEn ? e.hrefEn : (e.hrefZh || e.href);
  }

  function queryTokens(q) {
    var raw = q.trim().toLowerCase().split(/\s+/);
    var seen = {};
    var out = [];
    for (var i = 0; i < raw.length; i++) {
      if (!raw[i] || seen[raw[i]]) continue;
      seen[raw[i]] = true;
      out.push(raw[i]);
      if (out.length === 8) break;
    }
    return out;
  }

  function search(q) {
    var tokens = queryTokens(q);
    if (!tokens.length) return [];
    var phrase = q.trim().toLowerCase();
    var all = getIndex();
    var scored = [];
    for (var i = 0; i < all.length; i++) {
      var e = all[i];
      var ok = true;
      for (var j = 0; j < tokens.length; j++) {
        if (e._all.indexOf(tokens[j]) === -1) { ok = false; break; }
      }
      if (!ok) continue;
      var score = 0;
      for (j = 0; j < tokens.length; j++) {
        var tok = tokens[j];
        if (e._tz.indexOf(tok) !== -1) score += 42;
        if (e._te.indexOf(tok) !== -1) score += 36;
        if (e._bz.indexOf(tok) !== -1) score += 14;
        if (e._be.indexOf(tok) !== -1) score += 10;
        if (String(e.keys || '').toLowerCase().indexOf(tok) !== -1) score += 8;
      }
      if (e._tz === phrase || e._te === phrase) score += 120;
      else if (e._tz.indexOf(phrase) === 0 || e._te.indexOf(phrase) === 0) score += 60;
      else if (e._tz.indexOf(phrase) !== -1) score += 24;
      else if (e._te.indexOf(phrase) !== -1) score += 18;
      scored.push({ e: e, score: score, order: i });
    }
    scored.sort(function (a, b) {
      return b.score - a.score || a.order - b.order;
    });
    var out = [];
    for (i = 0; i < scored.length && i < 8; i++) out.push(scored[i].e);
    return out;
  }

  function quickEntries() {
    return getIndex().filter(function (e) { return e.quick; });
  }

  function hasAny(text, tokens) {
    var lower = text.toLowerCase();
    return tokens.some(function (token) { return lower.indexOf(token) !== -1; });
  }

  function shorten(text, max) {
    return text.length <= max ? text : text.slice(0, max).trim() + ' …';
  }

  function snippetFor(e, tokens) {
    var l = lang();
    var other = l === 'en' ? 'zh' : 'en';
    var text = e.body[l] || '';
    if (tokens.length && !hasAny(text, tokens) && e.body[other]) text = e.body[other];
    if (!text) return '';
    if (!tokens.length) return shorten(text, 150);
    var lower = text.toLowerCase();
    var at = -1;
    for (var i = 0; i < tokens.length; i++) {
      var found = lower.indexOf(tokens[i]);
      if (found !== -1 && (at === -1 || found < at)) at = found;
    }
    if (at === -1) return shorten(text, 150);
    var start = Math.max(0, at - 36);
    var end = Math.min(text.length, start + 150);
    if (start > 0 && /[a-z0-9]/i.test(text.charAt(start - 1)) && /[a-z0-9]/i.test(text.charAt(start))) {
      var space = text.indexOf(' ', start);
      if (space !== -1 && space < at) start = space + 1;
    }
    if (end < text.length) {
      var cut = text.lastIndexOf(' ', end);
      if (cut > at && cut > start) end = cut;
    }
    return (start > 0 ? '… ' : '') + text.slice(start, end).trim() + (end < text.length ? ' …' : '');
  }

  function escapeRe(value) {
    return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  var HTML_ESC = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' };
  function esc(value) {
    return String(value).replace(/[&<>"']/g, function (ch) { return HTML_ESC[ch]; });
  }

  function highlight(text, tokens) {
    if (!tokens.length) return esc(text);
    var sorted = tokens.slice().sort(function (a, b) { return b.length - a.length; });
    var re;
    try {
      re = new RegExp('(' + sorted.map(escapeRe).join('|') + ')', 'gi');
    } catch (err) {
      return esc(text);
    }
    var parts = String(text).split(re);
    var out = '';
    for (var i = 0; i < parts.length; i++) {
      out += i % 2 ? '<mark>' + esc(parts[i]) + '</mark>' : esc(parts[i]);
    }
    return out;
  }

  function labelOf(type) {
    if (type === 'feature') return t('功能', 'Feature');
    if (type === 'faq') return t('问答', 'FAQ');
    if (type === 'release') return t('版本', 'Release');
    if (type === 'resource') return t('资源', 'Resource');
    if (type === 'milestone') return t('里程碑', 'Milestone');
    return t('页面', 'Page');
  }

  function itemHtml(e, i, tokens) {
    var title = e.title[lang()] || e.title.zh || '';
    var snippet = snippetFor(e, tokens);
    return '<li class="search-result" role="option" id="site-search-option-' + i + '" aria-selected="false">' +
      '<a href="' + esc(hrefOf(e)) + '">' +
        '<span class="search-result-type">' + esc(labelOf(e.type)) + '</span>' +
        '<span class="search-result-title">' + highlight(title, tokens) + '</span>' +
        (snippet ? '<span class="search-result-snippet">' + highlight(snippet, tokens) + '</span>' : '') +
      '</a></li>';
  }

  function setActive(i, scroll) {
    var items = list.children;
    active = i;
    for (var j = 0; j < items.length; j++) {
      var on = j === i;
      items[j].classList.toggle('is-active', on);
      items[j].setAttribute('aria-selected', on ? 'true' : 'false');
    }
    if (i >= 0 && items[i]) {
      input.setAttribute('aria-activedescendant', items[i].id);
      if (scroll && items[i].scrollIntoView) items[i].scrollIntoView({ block: 'nearest' });
    } else input.removeAttribute('aria-activedescendant');
  }

  function render(q) {
    q = q || '';
    var tokens = queryTokens(q);
    var matched = tokens.length ? search(q) : quickEntries();
    results = matched;
    var html = '';
    for (var i = 0; i < matched.length; i++) html += itemHtml(matched[i], i, tokens);
    list.innerHTML = html;
    list.hidden = !matched.length;
    list.setAttribute('aria-label', tokens.length ? t('搜索结果', 'Search results') : t('快速跳转', 'Quick links'));
    empty.classList.toggle('is-visible', !matched.length);
    input.setAttribute('aria-expanded', matched.length ? 'true' : 'false');
    var n = matched.length;
    count.textContent = !n ? t('没有匹配结果', 'No results')
      : !tokens.length ? t('快速跳转', 'Quick links')
      : n === 1 ? t('1 个结果', '1 result') : t(n + ' 个结果', n + ' results');
    setActive(n ? 0 : -1, false);
  }

  function move(delta) {
    if (!results.length) return;
    var next = active < 0
      ? (delta > 0 ? 0 : results.length - 1)
      : (active + delta + results.length) % results.length;
    setActive(next, true);
  }

  function choose() {
    if (active < 0 || !results.length) return;
    var link = list.children[active] && list.children[active].querySelector('a');
    if (link) link.click();
  }

  function close() {
    if (!dialog) return;
    if (typeof dialog.close === 'function' && dialog.open) { dialog.close(); return; }
    dialog.removeAttribute('open');
    document.body.classList.remove('search-open');
    if (lastFocus && lastFocus.focus) lastFocus.focus();
    lastFocus = null;
  }
  function open() {
    if (!dialog) return;
    if (dialog.open) {
      input.focus();
      input.select();
      return;
    }
    if (document.body.classList.contains('nav-open')) {
      document.body.classList.remove('nav-open');
      var menu = document.querySelector('.menu-btn');
      if (menu) menu.setAttribute('aria-expanded', 'false');
    }
    lastFocus = document.activeElement === document.body ? trigger : (document.activeElement || trigger);
    input.value = '';
    render('');
    document.body.classList.add('search-open');
    if (typeof dialog.showModal === 'function') dialog.showModal();
    else dialog.setAttribute('open', '');
    input.focus();
  }

  function syncChrome() {
    if (!dialog) return;
    titleEl.textContent = t('搜索站点', 'Search the site');
    input.setAttribute('aria-label', t('搜索站点', 'Search the site'));
    input.placeholder = t('搜索功能、安装、FAQ、版本…', 'Search features, install, FAQ, releases…');
    closeBtn.setAttribute('aria-label', t('关闭搜索', 'Close search'));
    empty.textContent = t('没有找到匹配内容，试试“ORM”“Redis”或“安装”。', 'No matches. Try “ORM”, “Redis”, or “install”.');
    hint.innerHTML = '<kbd>↑</kbd><kbd>↓</kbd> ' + t('选择', 'select') + ' · <kbd>↵</kbd> ' +
      t('打开', 'open') + ' · <kbd>Esc</kbd> ' + t('关闭', 'close');
    trigger.setAttribute('aria-label', t('搜索站点', 'Search the site'));
    trigger.title = t('搜索', 'Search') + ' (' + shortcut + ')';
    render(input.value);
  }

  function buildDialog() {
    dialog = document.createElement('dialog');
    dialog.className = 'search-dialog'; dialog.id = 'site-search';
    dialog.setAttribute('aria-labelledby', 'site-search-title');
    dialog.innerHTML =
      '<div class="search-panel">' +
        '<h2 class="sr-only" id="site-search-title"></h2>' +
        '<div class="search-box" role="search">' +
          '<svg class="search-icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"></circle><path d="m20 20-4.2-4.2"></path></svg>' +
          '<input class="search-input" id="site-search-input" type="search" role="combobox" aria-autocomplete="list" aria-haspopup="listbox" aria-controls="site-search-results" aria-expanded="false" autocomplete="off" spellcheck="false" maxlength="120">' +
          '<button class="search-close" type="button"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18"></path></svg></button>' +
        '</div>' +
        '<div class="search-meta"><span class="search-count" id="site-search-count" aria-live="polite"></span><span class="search-hint" aria-hidden="true"></span></div>' +
        '<ul class="search-results" id="site-search-results" role="listbox"></ul>' +
        '<p class="search-empty" id="site-search-empty"></p>' +
      '</div>';
    document.body.appendChild(dialog);
    input = dialog.querySelector('.search-input');
    list = dialog.querySelector('.search-results');
    empty = dialog.querySelector('.search-empty');
    count = dialog.querySelector('.search-count');
    hint = dialog.querySelector('.search-hint');
    closeBtn = dialog.querySelector('.search-close');
    titleEl = dialog.querySelector('#site-search-title');
    closeBtn.addEventListener('click', close);
    dialog.addEventListener('click', function (e) { if (e.target === dialog) close(); });
    dialog.addEventListener('close', function () {
      document.body.classList.remove('search-open');
      if (lastFocus && lastFocus.focus) lastFocus.focus();
      lastFocus = null;
    });
    input.addEventListener('input', function () { render(input.value); });
    input.addEventListener('keydown', function (e) {
      if (e.key === 'ArrowDown') { e.preventDefault(); move(1); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); move(-1); }
      else if (e.key === 'Enter') { e.preventDefault(); choose(); }
      else if (e.key === 'Escape') { e.preventDefault(); close(); }
    });
    list.addEventListener('mousemove', function (e) {
      var item = e.target.closest && e.target.closest('.search-result');
      if (!item) return;
      for (var i = 0; i < list.children.length; i++) {
        if (list.children[i] === item && active !== i) setActive(i, false);
      }
    });
    list.addEventListener('click', function (e) { if (e.target.closest && e.target.closest('a')) close(); });
  }

  function injectTrigger() {
    trigger = document.createElement('button');
    trigger.type = 'button'; trigger.className = 'search-btn';
    trigger.setAttribute('aria-haspopup', 'dialog');
    trigger.setAttribute('aria-controls', 'site-search');
    trigger.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"></circle><path d="m20 20-4.2-4.2"></path></svg>';
    trigger.addEventListener('click', open);
    var langBtn = actions.querySelector('[data-lang-toggle]');
    actions.insertBefore(trigger, langBtn || actions.firstChild);
  }

  injectTrigger();
  buildDialog();
  syncChrome();

  document.addEventListener('beerust:lang', syncChrome);
  document.addEventListener('keydown', function (e) {
    if (e.defaultPrevented || e.isComposing) return;
    if ((e.metaKey || e.ctrlKey) && !e.altKey && (e.key === 'k' || e.key === 'K')) {
      e.preventDefault();
      open();
      return;
    }
    if (e.key !== '/' || e.metaKey || e.ctrlKey || e.altKey) return;
    var el = e.target, tag = (el && el.tagName || '').toLowerCase();
    if (el && (el.isContentEditable || tag === 'input' || tag === 'textarea' || tag === 'select')) return;
    if (dialog && dialog.open) return;
    e.preventDefault();
    open();
  });
})();
