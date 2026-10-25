/* Bee Rust site — changelog renderer */
(function () {
  'use strict';
  var data = window.BeeChangelog || [];
  var i18n = window.BeeI18n;
  var list = document.getElementById('log-list');
  var index = document.getElementById('log-index');
  if (!list || !index) return;
  var currentObserver = null;
  var t = function (zh, en) { return i18n ? i18n.t(zh, en) : zh; };
  var lang = function () { return i18n ? i18n.lang : 'zh'; };
  var slug = function (version) { return 'v' + version.replace(/\./g, '-'); };

  function render() {
    var l = lang();
    var changelogUrl = l === 'en'
      ? 'https://github.com/erikwang2013/bee-rust/blob/main/docs/CHANGELOG.md'
      : 'https://github.com/erikwang2013/bee-rust/blob/main/docs/CHANGELOG.zh.md';

    index.innerHTML = data.map(function (r) {
      return '<a href="#' + slug(r.version) + '" data-version="' + r.version + '">' +
        r.version + ' <span>' + r.date + '</span></a>';
    }).join('');

    list.innerHTML = data.map(function (r) {
      var d = r[l] || r.zh;
      var latest = r.tag === 'latest';
      var tagLabel = latest ? t('最新', 'LATEST') : r.tag;
      return '<article class="release' + (latest ? ' is-latest' : '') + '" id="' + slug(r.version) + '" data-reveal>' +
        '<div class="release-side">' +
          '<span class="release-version">' + r.version + '</span>' +
          '<time datetime="' + r.date + '">' + r.date + '</time>' +
          '<span class="release-tag">' + tagLabel + '</span>' +
        '</div>' +
        '<div class="release-body">' +
          '<h2>' + d.title + '</h2>' +
          '<p class="release-summary">' + d.summary + '</p>' +
          '<ul>' + d.items.map(function (item) { return '<li>' + item + '</li>'; }).join('') + '</ul>' +
          '<a class="text-link" href="' + changelogUrl + '" target="_blank" rel="noopener">' +
            t('在完整日志中查看', 'Open in full changelog') + ' <span aria-hidden="true">↗</span></a>' +
        '</div>' +
      '</article>';
    }).join('');

    if (window.BeeSite && window.BeeSite.observeReveals) window.BeeSite.observeReveals(list);
    bindIndexObserver();
    var title = l === 'en' ? 'Changelog · Bee Rust' : '版本日志 · Bee Rust';
    var description = l === 'en'
      ? 'Bee Rust release history, from the initial 1.0.3 workspace to the 1.6.1 release.'
      : 'Bee Rust 版本日志：从 1.0.3 到 1.6.1 的每次迭代，按版本查看新增、修复与清理。';
    if (window.BeeSite && window.BeeSite.applyMeta) {
      window.BeeSite.applyMeta(title, description, l === 'en' ? 'en_US' : 'zh_CN');
    } else {
      document.title = title;
      var meta = document.querySelector('meta[name="description"]');
      if (meta) meta.setAttribute('content', description);
    }
  }

  function bindIndexObserver() {
    if (currentObserver) currentObserver.disconnect();
    if (!('IntersectionObserver' in window)) return;
    var links = index.querySelectorAll('a');
    if (links.length) {
      links[0].classList.add('is-current');
      links[0].setAttribute('aria-current', 'true');
    }
    currentObserver = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        var id = entry.target.id;
        for (var i = 0; i < links.length; i++) {
          var active = links[i].getAttribute('href') === '#' + id;
          links[i].classList.toggle('is-current', active);
          if (active) links[i].setAttribute('aria-current', 'true');
          else links[i].removeAttribute('aria-current');
        }
      });
    }, { rootMargin: '-20% 0px -68% 0px', threshold: 0 });
    var releases = list.querySelectorAll('.release');
    for (var i = 0; i < releases.length; i++) currentObserver.observe(releases[i]);
  }

  render();
  document.addEventListener('beerust:lang', render);
})();
