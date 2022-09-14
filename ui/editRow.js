
let nameDefault = "";
let activityDefault = "";
let variousActivities = "# various activity types #";
let variousNames = "# various names #";
let variousActivitiesOption = document.createElement('option');
variousActivitiesOption.value = "# various activity types #";
variousActivitiesOption.innerHTML = "# various activity types #";

function editRow() {
    let overlay = document.getElementById("table-overlay");
    let button = document.getElementById("table-button-edit");
    let nameField = document.getElementById("name-input");
    document.getElementById("invalid-characters-alert").style.display = "";
    nameField.style.color = ""; // reset color if changed with last focus
    nameField.style.borderColor = ""
    let activityField = document.getElementById("activity-input");
    activityField.style.color = ""; // reset color if changed with last focus
    if (overlay.style.display !== "flex") {
        let warning = document.getElementById("multiple-rows-warning");
        if (selected_rows.length > 1) {
            warning.style.display = "";

            // check for every editable attribute if it is the same across all selected entries
            let singleActivity = true;
            let singleName = true;
            let lastActivity = row_objects[selected_rows[0]]._type;
            let lastName = row_objects[selected_rows[0]].name;
            for (let u of selected_rows) {
                if (row_objects[u]._type !== lastActivity) {
                    singleActivity = false;
                }
                if (row_objects[u].name !== lastName) {
                    singleName = false;
                }
            }
            if (singleActivity) {
                if (document.getElementById("activity-input").contains(variousActivitiesOption)) {
                    document.getElementById("activity-input").removeChild(variousActivitiesOption);
                }
                activityDefault = row_objects[selected_rows[0]]._type;
            } else {
                document.getElementById("activity-input").appendChild(variousActivitiesOption);
                activityDefault = variousActivities;
            }
            if (singleName) {
                nameDefault = row_objects[selected_rows[0]].name;
            } else {
                nameDefault = variousNames;
            }

        } else {
            warning.style.display = "none";
            activityDefault = row_objects[selected_rows[0]]._type;
            nameDefault = row_objects[selected_rows[0]].name;
        }
        activityField.value = activityDefault;
        nameField.value = nameDefault;
        overlay.style.display = "flex";
        button.style.opacity = "0.6";
        button.style.pointerEvents = "none";
    } else {
        console.log("!Should not be able to click edit button while flex window open");
        overlay.style.display = "none";
    }
}

function cancelEditRow() {
    let overlay = document.getElementById("table-overlay");
    let button = document.getElementById("table-button-edit");
    if (overlay.style.display !== "none") {
        overlay.style.display = "none";
        button.style.opacity = "1.0";
        button.style.pointerEvents = "";
    }
}

function saveEditRow() {
    let activity = document.getElementById("activity-input").value;
    let name = document.getElementById("name-input").value;
    if (activity === activityDefault && name === nameDefault) {
        console.log("nothing to change");
        cancelEditRow();
        return;
    }
    if (selected_rows.length > 1) {
        // TODO
        for (let u of selected_rows) {
            let activityUpdate = activity;
            if (activity === activityDefault) {
                activityUpdate = row_objects[u]._type;
            }
            let nameUpdate = name;
            if (name === nameDefault) {
                nameUpdate = row_objects[u].name;
            }
            invoke("save_track_changes", { ulid: u, name: nameUpdate, activity: activityUpdate })
        }
        reload_table();
        cancelEditRow();
    } else if (selected_rows.length == 1) {
        // check for every entry if input is valid
        let ulid = selected_rows[0];
        invoke("save_track_changes", { ulid: ulid, name: name, activity: activity })
        .then(() => {
            reload_table();
            var rows = table_body.querySelectorAll("tr");
            rows.forEach(row => {
                if (row.querySelectorAll("td")[0].innerHTML == ulid) {
                    clear_table_selection();
                    toggle_row_selection(ulid);
                    cancelEditRow();
                    return;
                }
            });
            cancelEditRow();
        });
    }
}

// TODO: maybe add more restrictions
function inputIsValid(input) {
    if (input.includes("\"")) {
        return false
    }
    return true
}

function focusInput(field) {
    let inputfield;
    if (field === 'activity') {
        inputfield = document.getElementById("activity-input");
        inputfield.style.color = "#000";
    } else if (field === 'name') {
        inputfield = document.getElementById("name-input");
        inputfield.style.color = "#000";
        if (selected_rows.length > 1) {
            if (inputfield.value === variousNames) {
                inputfield.value = "";
            }
        }
    }
}

function checkInput(field) {
    console.log("check input");
    let inputField;
    let button = document.getElementById('save-row-edit');
    if (field === 'activity') {
        inputField = document.getElementById("activity-input");
        if (inputField.value === activityDefault) {
            inputField.style.color = "";
        } else {
            inputField.style.color = "#000";
        }
    } else if (field === 'name') {
        inputField = document.getElementById("name-input");

        if (!inputIsValid(inputField.value)) {
            document.getElementById("invalid-characters-alert").style.display = "block";
            inputField.style.borderColor = "#ff6473";
            button.style.opacity = "0.6";
            button.style.pointerEvents = "none";
            return;
        }

        if (inputField.value === "" || inputField.value === nameDefault) {
            inputField.value = nameDefault;
            inputField.style.color = "";
        }

        document.getElementById("invalid-characters-alert").style.display = "none";
        inputField.style.borderColor = "";
        if (allInputsValid()) {
            button.style.opacity = "1.0";
            button.style.pointerEvents = "";
        }
    }
}

function allInputsValid() {
    // check all input fields if valid
    if (!inputIsValid(document.getElementById("name-input").value)) {
        return false;
    }
    return true
}