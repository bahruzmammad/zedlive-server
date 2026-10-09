const status = document.getElementById("status");
const ajaxStatus = document.getElementById("ajax-status");

status.textContent = window.jQueryTest
  ? "JavaScript + jQuery-compatible helper loaded ✅"
  : "JavaScript loaded";

$.ajax({
  url: "/api/data.json",
  success(data) {
    ajaxStatus.textContent = `AJAX JSON loaded: ${data.message} ✅`;
  },
});
