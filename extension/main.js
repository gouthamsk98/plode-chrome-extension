// Copyright 2013 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

let message;
let port = null;

function appendMessage(text) {
  document.getElementById("response").innerHTML += "<p>" + text + "</p>";
}

function updateUiState() {
  if (port) {
    document.getElementById("connect-button").style.display = "none";
    document.getElementById("input-text").style.display = "block";
    document.getElementById("send-message-button").style.display = "block";
  } else {
    document.getElementById("connect-button").style.display = "block";
    document.getElementById("input-text").style.display = "none";
    document.getElementById("send-message-button").style.display = "none";
  }
}

function sendNativeMessage() {
  message = document.getElementById("input-text").value;
  port.postMessage(message);
  appendMessage("Sent message: <b>" + JSON.stringify(message) + "</b>");
}

function onNativeMessage(message) {
  console.log("res message", message);
  appendMessage("Received message: <b>" + message + "</b>");
}

function onDisconnected() {
  appendMessage("Failed to connect: " + chrome.runtime.lastError.message);
  port = null;
  updateUiState();
}

function connect() {
  const hostName = "com.plode_mass_storage.native";
  appendMessage("Connecting to native messaging host <b>" + hostName + "</b>");
  port = chrome.runtime.connectNative(hostName);
  port.onMessage.addListener(onNativeMessage);
  port.onDisconnect.addListener(onDisconnected);

  updateUiState();
}

document.addEventListener("DOMContentLoaded", function () {
  connect();
  document.getElementById("connect-button").addEventListener("click", connect);
  document
    .getElementById("send-message-button")
    .addEventListener("click", sendNativeMessage);
  updateUiState();
});
