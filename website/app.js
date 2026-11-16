(function(){
"use strict";
var $=function(s,r){return (r||document).querySelector(s)},$$=function(s,r){return Array.prototype.slice.call((r||document).querySelectorAll(s))};
var reduce=window.matchMedia("(prefers-reduced-motion: reduce)").matches||document.body.classList.contains("reduce-motion");
var REPO="monk81694-crypto/cinder";
try{var a=localStorage.getItem("cinder-accent");if(a)document.body.dataset.accent=a}catch(e){}
function toast(m){var h=$(".toasts");if(!h){h=document.createElement("div");h.className="toasts";h.setAttribute("role","status");document.body.appendChild(h)}var t=document.createElement("div");t.className="toast";t.textContent=m;h.appendChild(t);setTimeout(function(){t.remove()},3200)}
// nav
var nt=$(".nav-toggle"),nl=$(".nav ul");
if(nt&&nl)nt.addEventListener("click",function(){var o=nl.classList.toggle("open");nt.setAttribute("aria-expanded",o)});
// reveal
var rv=$$(".reveal");
if(reduce||!("IntersectionObserver" in window))rv.forEach(function(e){e.classList.add("in")});
else{var io=new IntersectionObserver(function(es){es.forEach(function(e){if(e.isIntersecting){e.target.classList.add("in");io.unobserve(e.target)}})},{threshold:.12});rv.forEach(function(e){io.observe(e)})}
// counters
$$("[data-count]").forEach(function(el){var end=+el.dataset.count;if(reduce){el.textContent=end;return}el.textContent="0";var o=new IntersectionObserver(function(es){if(!es[0].isIntersecting)return;o.disconnect();var t0=performance.now();(function f(t){var p=Math.min((t-t0)/900,1);el.textContent=Math.round(end*(1-Math.pow(1-p,3)));if(p<1)requestAnimationFrame(f)})(t0)});o.observe(el)});
// parallax
var bg=$(".hero-bg");
if(bg&&!reduce)window.addEventListener("scroll",function(){bg.style.transform="translateY("+Math.min(window.scrollY*.25,120)+"px)"},{passive:true});
// carousel
$$("[data-carousel]").forEach(function(c){var n=$("[data-next]",c.parentNode),p=$("[data-prev]",c.parentNode);function step(d){c.scrollBy({left:d*c.clientWidth*.8,behavior:reduce?"auto":"smooth"})}if(n)n.onclick=function(){step(1)};if(p)p.onclick=function(){step(-1)}});
// OS detect + release
var ua=navigator.userAgent,os=/Win/.test(ua)?"win":/Mac/.test(ua)?"mac":/Linux|X11/.test(ua)?"linux":"";
if(os){var m=$('[data-os="'+os+'"]');if(m){m.classList.add("match");var b=$(".badge-slot",m);if(b)b.innerHTML='<span class="badge badge-active">Your system</span>'}}
var RE={win:/\.exe$/i,mac:/\.dmg$/i,linux:/\.AppImage$/i};
var PAGE="https://github.com/"+REPO+"/releases/latest";
$$("[data-os] a.btn").forEach(function(a){a.href=PAGE});
if($("[data-os]")){
 fetch("https://api.github.com/repos/"+REPO+"/releases/latest",{headers:{Accept:"application/vnd.github+json"}}).then(function(r){if(!r.ok)throw new Error(r.status);return r.json()}).then(function(rel){
  var v=$("#rel-version");if(v){v.textContent=rel.tag_name;v.classList.remove("skeleton")}
  $$("[data-os]").forEach(function(c){var re=RE[c.dataset.os],as=(rel.assets||[]).filter(function(x){return re.test(x.name)})[0];if(as){$("a.btn",c).href=as.browser_download_url;var s=$(".size",c);if(s)s.textContent=(as.size/1048576).toFixed(0)+" MB"}});
  var sha=$("#sha");var mm=((rel.body||"").match(/\b[a-f0-9]{64}\b/i)||[])[0];if(sha)sha.textContent=mm||"Not published with this release. Compare against the file hash shown by your OS tools.";if(sha)sha.classList.remove("skeleton");
 }).catch(function(){var v=$("#rel-version");if(v){v.textContent="see GitHub";v.classList.remove("skeleton")}var s=$("#sha");if(s){s.textContent="Release info unavailable. Use the GitHub releases page.";s.classList.remove("skeleton")}});
}
var cp=$("#copy-sha");
if(cp)cp.addEventListener("click",function(){var t=$("#sha").textContent;if(!/^[a-f0-9]{64}$/i.test(t)){toast("No checksum to copy");return}(navigator.clipboard?navigator.clipboard.writeText(t):Promise.reject()).then(function(){cp.textContent="Copied";toast("Checksum copied");setTimeout(function(){cp.textContent="Copy"},1800)},function(){toast("Copy failed. Select the text manually.")})});
// accordion
$$(".acc button").forEach(function(b){var p=b.nextElementSibling;b.addEventListener("click",function(){var o=b.getAttribute("aria-expanded")==="true";b.setAttribute("aria-expanded",!o);p.style.maxHeight=o?"0":p.scrollHeight+"px"})});
// toc
var tl=$$(".toc a");
if(tl.length&&"IntersectionObserver" in window){var so=new IntersectionObserver(function(es){es.forEach(function(e){if(e.isIntersecting)tl.forEach(function(a){a.classList.toggle("active",a.getAttribute("href")==="#"+e.target.id)})})},{rootMargin:"-30% 0px -60% 0px"});$$(".docs section[id]").forEach(function(s){so.observe(s)})}
// command palette
var pal=$("#palette");
if(pal){var inp=$("input",pal),list=$(".cmd-list",pal);
 var cmds=$$(".nav ul a").map(function(a){return{t:a.textContent,h:a.getAttribute("href")}});
 ["ember","mint","violet","glacier","rose"].forEach(function(x){cmds.push({t:"Accent: "+x,a:x})});
 function draw(q){list.innerHTML="";cmds.filter(function(c){return c.t.toLowerCase().indexOf(q.toLowerCase())>-1}).forEach(function(c){var li=document.createElement("li"),a=document.createElement("a");a.textContent=c.t;a.href=c.h||"#";a.onclick=function(e){if(c.a){e.preventDefault();document.body.dataset.accent=c.a;try{localStorage.setItem("cinder-accent",c.a)}catch(x){}close();toast("Accent set to "+c.a)}};li.appendChild(a);list.appendChild(li)})}
 function open(){pal.classList.add("open");inp.value="";draw("");inp.focus()}function close(){pal.classList.remove("open")}
 inp.addEventListener("input",function(){draw(inp.value)});
 pal.addEventListener("click",function(e){if(e.target===pal)close()});
 document.addEventListener("keydown",function(e){if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==="k"){e.preventDefault();pal.classList.contains("open")?close():open()}else if(e.key==="Escape")close()});
 var pb=$("#open-palette");if(pb)pb.addEventListener("click",open);
}
// launcher install buttons
$$(".launcher-install").forEach(function(btn){btn.addEventListener("click",function(e){e.preventDefault();var loader=btn.dataset.loader;toast("Opening "+loader+" installer...");window.open("https://github.com/monk81694-crypto/cinder/releases/latest","_blank")})});

})();
