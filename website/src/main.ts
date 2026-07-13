import {
  ArrowRight,
  Check,
  Download,
  ExternalLink,
  GitFork as Github,
  Languages,
  Menu,
  X,
  createIcons,
} from "lucide";

document.documentElement.classList.add("js");

createIcons({
  icons: { ArrowRight, Check, Download, ExternalLink, Github, Languages, Menu, X },
  attrs: { "aria-hidden": "true", "stroke-width": "1.8" },
});

const menuButton = document.querySelector<HTMLButtonElement>("[data-menu-toggle]");
const navigation = document.querySelector<HTMLElement>("[data-site-navigation]");

function setMenu(open: boolean): void {
  if (!menuButton || !navigation) return;
  menuButton.setAttribute("aria-expanded", String(open));
  navigation.toggleAttribute("data-open", open);
  document.body.toggleAttribute("data-menu-open", open);
}

menuButton?.addEventListener("click", () => {
  setMenu(menuButton.getAttribute("aria-expanded") !== "true");
});

navigation?.addEventListener("click", (event) => {
  if ((event.target as Element).closest("a")) setMenu(false);
});

window.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && menuButton?.getAttribute("aria-expanded") === "true") {
    setMenu(false);
    menuButton.focus();
  }
});

const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
if (!reduceMotion && "IntersectionObserver" in window) {
  const observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      entry.target.setAttribute("data-visible", "");
      observer.unobserve(entry.target);
    }
  }, { rootMargin: "0px 0px -8%", threshold: 0.12 });
  document.querySelectorAll("[data-reveal]").forEach((element) => observer.observe(element));
} else {
  document.querySelectorAll("[data-reveal]").forEach((element) => element.setAttribute("data-visible", ""));
}
