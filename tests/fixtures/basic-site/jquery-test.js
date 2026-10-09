window.jQueryTest = true;
window.$ = window.$ || function (selector) { return document.querySelector(selector); };
window.$.ajax = function (options) {
  return fetch(options.url).then((response) => response.json()).then(options.success);
};
