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
			.then((_) => {
				icon_elem.innerHTML = "done";
				icon_elem.classList.remove("spin");
				task_inputs.set(form.getAttribute("id"), form_data.get("task"));
				task_on_change(form.querySelector("input[type=text]"));
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
					.then(() => {
						input.checked = false;
						input.parentNode.parentNode.remove();
					});
			}
		}, 3000)
	} else {
		input.parentNode.parentNode.querySelector("form.task-form input[type=text]").classList.remove("crossed-out");
	}
}