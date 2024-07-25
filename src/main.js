const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

async function generateHtml(inputMd) {
  return await invoke("get_html_from_markdown", { input: inputMd });
}

function debouncedConvert(wait) {
  let timeout;

  return function () {
    const later = () => {
      clearTimeout(timeout);
      
      console.log("We were here.");
      let input = document.querySelector("#editor").value;
      generateHtml(input).then(output => {
        document.querySelector("#canvas").innerHTML = output;
      });
    }

    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

window.addEventListener("DOMContentLoaded", () => {
  const updateMarkdown = debouncedConvert(500);
  document.querySelector("#editor").addEventListener("input", updateMarkdown);
});
