var DEFAULT_BBXML = "<bbxml>\n" +
    "  <templates>\n" +
    "  </templates>\n" +
    "\n" +
    "  <classes>\n" +
    "  </classes>\n" +
    "\n" +
    "  <body>\n" +
    "  </body>\n" +
    "</bbxml>";
var BBXML_INPUT = "<div>" +
    "<textarea id=\"bbxml-input\" class=\"textCtrl MessageEditor\"></textarea>" +
    "</div>";
var BBXML_BUTTON = "<input type=\"button\" value=\"🠉 Convert to BBCode 🠉\" id=\"bbxml-button\" class=\"button JsOnly\" style=\"display: block; margin: 10px auto;\" />";
var BBCODER_SCRIPT = "https://www.lyros.net/bbcoder.php";

function init() {
    // Locate the message editor
    var editor = document.getElementById("ctrl_message");
    if (!editor) {
        // This is not a page with an editor, abort
        console.log("[bbcoder] DEBUG: No editor box found.");
        return;
    }
    editor.insertAdjacentHTML("afterend", BBXML_BUTTON + BBXML_INPUT);

    // ADD the BBXML editor
    var bbxmlInput = document.getElementById("bbxml-input");
    CodeMirror.defineMode("bbxml", function(config) {
        return CodeMirror.multiplexingMode(
            CodeMirror.getMode(config, "application/xml"),
            {
                open: "<class ", close: "</class>",
                mode: CodeMirror.getMode(config, "text/css"),
                delimStyle: "delimit"
            }
        );
    });
    var bbxmlEditor = CodeMirror.fromTextArea(document.getElementById("bbxml-input"), {
        mode: "application/xml",
        autoCloseTags: true,
        lineNumbers: true,
        lineWrapping: true
    });
    bbxmlEditor.setValue(DEFAULT_BBXML);

    // Add the BBXML conversion button
    var bbxmlButton = document.getElementById("bbxml-button");
    bbxmlButton.addEventListener("click", function() {
        var updateRequest = new XMLHttpRequest();
        updateRequest.onreadystatechange = function() {
            if (this.readyState === 4 && this.status === 200) {
                if (!this.responseText) {
                    alert("An unknown error occurred during conversion.");
                    return;
                }
                var response = JSON.parse(this.responseText);
                if (response.status === "success") {
                    editor.innerHTML = response.bbcode;
                } else {
                    alert(response.message);
                }
            }
        };
        updateRequest.open("POST", BBCODER_SCRIPT);
        var formData = new FormData();
        formData.append("src", bbxmlEditor.getValue());
        updateRequest.send(formData);
    });
}

init();
