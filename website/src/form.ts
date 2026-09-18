import extractBeasts from "./lib.rs";

function $<T = HTMLElement>(selector: `#${string}`): T {
  return document.querySelector(selector) as T;
}

const $form = $("#form");
const $selectPages = $<HTMLSelectElement>("#predefined-page-ranges");
const $pages = $<HTMLInputElement>("#pages");

export function setup() {
  $selectPages.onchange = () => {
    let selectedPages = $selectPages.value;
    $pages.value = "";
  }
  $pages.oninput = () => {
    let error = extractBeasts.validate_pages($pages.value);
    $pages.setCustomValidity(error ?? "");
  }
  $form.style.removeProperty("display");
}
