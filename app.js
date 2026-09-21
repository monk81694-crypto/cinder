// Cinder website — shared behaviour. No framework, no dependencies.
// 1) mobile nav toggle  2) active-link highlight  3) footer year.
(function () {
  "use strict";
  var toggle = document.querySelector(".nav-toggle");
  var navList = document.getElementById("site-nav");
  if (toggle && navList) {
    toggle.addEventListener("click", function () {
      var open = navList.classList.toggle("open");
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
    });
    navList.addEventListener("click", function (event) {
      if (event.target && event.target.tagName === "A" && navList.classList.contains("open")) {
        navList.classList.remove("open");
        toggle.setAttribute("aria-expanded", "false");
      }
    });
  }
  try {
    var page = window.location.pathname.split("/").pop() || "index.html";
    var links = document.querySelectorAll(".nav-list a[href]");
    for (var i = 0; i < links.length; i++) {
      var href = links[i].getAttribute("href");
      if (!href || href.charAt(0) === "#" || /^https?:/i.test(href)) { continue; }
      var target = href.split("/").pop().split("#")[0];
      if (target === page) { links[i].setAttribute("aria-current", "page"); }
    }
  } catch (err) { /* nav stays usable without highlight */ }
  var year = document.getElementById("year");
  if (year) { year.textContent = String(new Date().getFullYear()); }
})();
