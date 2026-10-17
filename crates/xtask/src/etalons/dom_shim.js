// Минимальный DOM для исполнения скрипта дизайн-книги вне браузера: элементы хранят innerHTML,
// таймеры не запускаются, медиазапросы не совпадают — книга рисует ровно начальный кадр каждого макета.
var __elements = new Map();
function __element(id) {
  return {
    id: id, innerHTML: '', textContent: '', style: {}, dataset: {}, children: [], onclick: null,
    appendChild: function (child) { this.children.push(child); return child; },
    addEventListener: function () {},
    setAttribute: function () {},
    closest: function () { return null; },
    classList: { add: function () {}, remove: function () {} },
  };
}
var document = {
  getElementById: function (id) {
    if (!__elements.has(id)) __elements.set(id, __element(id));
    return __elements.get(id);
  },
  createElement: function () { return __element(''); },
  querySelectorAll: function () { return []; },
};
var window = { matchMedia: function () { return { matches: false }; } };
function setInterval() { return 0; }
function clearInterval() {}
function __etalonScreens(ids) {
  var screens = {};
  for (var i = 0; i < ids.length; i++) screens[ids[i]] = document.getElementById('scr-' + ids[i]).innerHTML;
  return JSON.stringify(screens);
}
