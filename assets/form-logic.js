let task_forms = document.querySelectorAll("form.task-form");
let task_inputs = new Map();

for (let i = 0; i < task_forms.length; i++) {
	let form = task_forms[i];
	let input_field = form.querySelector("input[type=text]");
	task_inputs.set(form.getAttribute("id"), input_field.value);
	form.addEventListener("submit", (e) => {
		set_task(form);
		e.preventDefault();
	})
}

let signup_form = document.querySelector("#signup-form");
if(signup_form != null) {
	signup_form.addEventListener("submit", (e) => {
		let form_data = new FormData(signup_form);
		let uname = form_data.get("username");
		let pass1 = form_data.get("password");
		let pass2 = form_data.get("password-validate");
		let error_msg = signup_form.querySelector(".form-error-msg");
		if(pass1 != pass2) {
			if(error_msg != null) { // Should never be null
				error_msg.innerHTML = "Error: Passwords to not match"
			}
			e.preventDefault();
		} else if(username == null || username.length == 0) {
			if(error_msg != null) { // Should never be null
				error_msg.innerHTML = "Error: Username cannot be empty"
			}
			e.preventDefault();
		} else if(pass1 == null || pass1.length == 0) {
			if(error_msg != null) { // Should never be null
				error_msg.innerHTML = "Error: Password cannot be empty"
			}
			e.preventDefault();
		}
	})
}

function task_on_change(input) {
	let form = input.parentNode;
	if(task_inputs.get(form.getAttribute("id")) != input.value) {
		if(!input.classList.contains("task-updated")) {
			input.classList.add("task-updated");
		}
	} else {
		if(input.classList.contains("task-updated")) {
			input.classList.remove("task-updated");
		}
	}
}

async function set_task(form) {
	let form_data = new FormData(form);

	if(task_inputs.get(form.getAttribute("id")) != form_data.get("task")) {
		let icon_elem = form.querySelector(".material-symbols-outlined");
		icon_elem.innerHTML = "progress_activity";
		icon_elem.classList.add("spin");

		fetch("/set_task", { method: "POST", body: form_data, redirect: "manual" })
			.then((response) => response.text())
			.then((response_body) => {
				if(response_body == "SUCCESS") {
					icon_elem.innerHTML = "done";
					icon_elem.classList.remove("spin");
					task_inputs.set(form.getAttribute("id"), form_data.get("task"));
					task_on_change(form.querySelector("input[type=text]"));
				} else {
					alert(response_body);
				}
			})
	}
}

function add_task() {
	let id = 0;

	let task_forms = document.querySelectorAll("form.task-form");

	for (let i = 0; i < task_forms.length; i++) {
		const form = task_forms[i];
		let fid_str = form.getAttribute("id");
		let fid = parseInt(fid_str.slice(5));
		if(fid >= id) {
			id = fid + 1;
		}
	}

	let task_elem = document.querySelector("#task_template").content.cloneNode(true);
	let task_form = task_elem.querySelector("form");
	let task_hidden_input = task_elem.querySelector("input[type=hidden]");
	let task_text = task_elem.querySelector("input[type=text]");

	let task_id = `task-${id}`
	task_form.setAttribute("id", task_id);
	task_hidden_input.value = task_id;

	let tasks_area = document.querySelector("main > section");
	tasks_area.appendChild(task_elem);
	task_form.addEventListener("submit", (e) => {
		set_task(task_form);
		e.preventDefault();
	})
	task_text.focus();
	task_on_change(task_text);
}

async function done_task_on_change(input) {
	let timer;
	clearTimeout(timer);

	if(input.checked) {
		input.parentNode.parentNode.querySelector("form.task-form input[type=text]").classList.add("crossed-out");

		timer = setTimeout(() => {
			if(input.checked) {
				let form = input.parentNode.parentNode.querySelector("form.task-form");
				let form_data = new FormData(form);

				fetch("/remove_task", { method: "POST", body: form_data, redirect: "manual" })
					.then((response) => response.text())
					.then((response_body) => {
						if(response_body == "SUCCESS") {
							input.checked = false;
							input.parentNode.parentNode.remove();
						} else {
							alert(response_body);
						}
					});
			}
		}, 3000)
	} else {
		input.parentNode.parentNode.querySelector("form.task-form input[type=text]").classList.remove("crossed-out");
	}
}