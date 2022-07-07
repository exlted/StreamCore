// #3 - Dockerize everything (Allow for docker builds)

window.global = {};

let protocol = "ws://";
if (location.protocol == "https:") {
  protocol = "wss://"
}

let socket = new WebSocket(protocol + location.host);
socket.onopen = function(e) {
  //alert("[open] Connection established");
  //alert("Sending to server");
  //socket.send("My name is John");
};

socket.onmessage = function(event) {
  //alert(`[message] Data received from server: ${event.data}`);
  // Send an event to our message holder to create a new message
  // provide the JSON (event.data) to whoever implements the display function
  //   If nobody is implementing a display function, we need to handle it in our default?
  let innerEvent = new CustomEvent("messageRecieved", {
    detail: JSON.parse(event.data)
  })
  document.dispatchEvent(innerEvent);
};

socket.onclose = function(event) {
  if (event.wasClean) {
    //alert(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
  } else {
    // e.g. server process killed or network down
    // event.code is usually 1006 in this case
    // alert('[close] Connection died');
    // Notify the user that something broke
  }
};

socket.onerror = function(error) {
  // alert(`[error] ${error.message}`);
  // Notify the user that something broke
};

function loadDefaultCust() {
  const head = document.getElementsByTagName("head")[0];
  let css = document.createElement("link");
  css.setAttribute("rel", "stylesheet");
  css.setAttribute("href", "/default.css");

  let script = document.createElement("script");
  script.setAttribute("type", "text/javascript");
  script.setAttribute("src", "/default.js")

  head.appendChild(css);
  head.appendChild(script);
}

function loadCust(customizationIndex, customization) {
  const head = document.getElementsByTagName("head")[0];
  customizationIndex.forEach(includeFile => {
    if (includeFile.elementType == "html") {
      let body = document.getElementsByTagName("body")[0];

      fetch("/cust/" + customization + "/" + includeFile.url)
      .then(response=> response.text())
      .then(text=> body.innerHTML += text);
      
      return;
    }
    if (includeFile.elementType == "json") {
      fetch("/cust/" + customization + "/" + includeFile.url)
      .then(response=> response.text())
      .then(text => {
        const data = JSON.parse(text)
        if (includeFile.cssVars) {
          const root = document.querySelector(":root");
  
          for (const property in data) {
            root.style.setProperty("--" + property, data[property]);
          }
        } else {
          window.global[includeFile.jsid] = data;
        }
      });
      return;
    }
    let element = document.createElement(includeFile.elementType);
    for (let attr in includeFile.includeAttrs) {
      if (includeFile.includeAttrs.hasOwnProperty(attr)){
        let attrText = includeFile.includeAttrs[attr].text;
        if (includeFile.includeAttrs[attr].prepend) {
          attrText = "/cust/" + customization + "/" + attrText;
        }

        element.setAttribute(attr, attrText);
      }
    }
    head.appendChild(element);
  });
}

const queryString = window.location.search;
const parms = new URLSearchParams(queryString);
let customization = parms.get("customization");
let getDefault = false;
if (customization == null) {
  loadDefaultCust();
} else {
  // Query the server for a customization
  //   if the customization exists, load it
  //   otherwise fall back to loading the default customization and display a message

  // In our customization file, we need a "includes" file to tell it how to include the items
  //   This means a JSON file that gets sent back when querying the server for customization details

  let xhr = new XMLHttpRequest();
  let url = "/cust/" + customization + "/index.json";
  xhr.open("GET", url, true);

  xhr.onreadystatechange = function() {
    if (this.readyState == 4) {
      if (this.status == 200) {
        loadCust(JSON.parse(this.responseText), customization);
        setTimeout(() => {
          let event = new CustomEvent("onWidgetLoad", {});
          window.dispatchEvent(event);
        }, 1000);
      } else {
        alert("Loading of customization [" + customization + "] failed. Please check your configuration");
        loadDefaultCust();
      }
    }
  }

  xhr.send();
}