# HTUI - HTTP Client for terminal

A keyboard-first http client with a Text-based User Interface.
Designed as a developer-friendly alternative to GUI tools like Postman/Insomnia, but in your terminal.
Run, save and organize HTTP requests across multiple projects.

## Why htui?
- 📡 Run HTTP requests quickly (GET, POST, PUT, DELETE, …)
- 🗂️ Multi-project support
- 📂 Collections of requests inside each project
- ↩️ Undo/Redo everywhere (deletions, mutations, creations)
- 🖥️ Clean TUI — navigate projects, collections, and requests with keyboard
- 🎨 Custom themes & keybindings
- ⚡ 100% terminal — no mouse required


## Quick Start
Run
```bash
  htui
  htui "[project name]"
```

## Layout sections
  * Opened Projects: Open, remove and edit projects
  * Collections sidebar: Show project collections & requests
  * Method url: Edit the http method and url
  * Request builder: Tabs section, where edit the params, headers & body
  * Response viewer: Show the request response


## Configuration
Htui can be customized via a toml file located on `~/.config/htui/config.toml` on (linux).

File example:
```toml
  [theme]
  ui =  { fg = "#D8DEE9",  bg = "#2E3440" }
  tab = "#81A1C1"
  tab_highlight = "#88C0D0"
  selection = { fg = "#2E3440", bg = "#434C5E" }
  dropdown = { fg = "#D8DEE9", bg = "#3B4252" }
  dropdown_highlight = { fg = "#ECEFF4", bg = "#5E81AC" }
  border = "#4C566A"
  border_focus = "#8FBCBB"

  [keymap.global]
  # Change the section focused
  next_focus = "Tab"
  previous_focus = "Shift+BackTab"

  # Keys used for movements through lists, table (index positions)
  move_down  = "j"
  move_up = "k"
  move_left = "h"
  move_right = "l"

  # Change the tab focused
  next_tab = "Shift+L"
  previous_tab = "Shift+H"

  # Send request
  send_request = "Alt+Enter"

  # Save the project locally by projectId
  save_project = "Alt+s"


  # Keys used for undo/redo actions
  undo = "Alt+u"
  redo = "Alt+y"

  # Close the popup, without apply changes
  close_popup = "Esc"

  # Close and apply changes
  submit_popup = "Enter"

  [keymap.app]
  # Open project selector
  search_project = "Alt+Shift+N"
  # Close current project (not save)
  close_project = "Alt+Shift+D"
  # Select next project
  next_project = "Alt+Shift+L"
  # Select previous project
  previous_project = "Alt+Shift+H"
  # Open a input for enter new project name
  rename_project = "Alt+Shift+R"


  [keymap.table]
  # Create new param
  new = "n"
  # Delete current param (row)
  delete = "d"
  # Edit current cell (inside a row param)
  edit = "e"


  [keymap.collections]
  # Delete the cursor index collection/request
  delete = "d"
  # Edit the cursor index collection/request name
  edit = "e"
  # Select the request (showed in the right area)
  select_request = "Enter"
  # Open popup input for collection name
  create_collection = "c"
  # Open popup input for request name
  create_request = "r"

  [keymap.method_url]
  # Opent the http method selector
  open_dropdown = "Enter"

  [keymap.request_builder]
  # Opent the Body type selector 
  open_dropdown = "Shift+O"
```

## Data Storage
The program manage these two json file as storage:
* Mapping file: `~/.local/share/htui/store/mapping.json`. A list of projects with only: Id, Name. Used for open project selector
* Project file: `~/.local/share/htui/store/[project_id].json`. A project structure stored as json. 


## ROADMAP
  - [ ] Open file selector for (download, select file to upload)
  - [ ] Improve the sending functionality for the request
  - [ ] Create a input/text own editor component
  - [*] Allow user select the path where save the bynary response (save file)
  - [-] Add copy to clipboard
  - [ ] Improve the UI
    - [ ] HTTP method colors (collections sidebar)
    - [ ] Scroll bar
    - [ ] Overlay messages (logs)
    - [x] Open external editors for data editing
  - [ ] Command footer bar, for show available key actions per layout section
  - [ ] Environment variables
  - [ ] Authentication, Cookies and Query request editing support
