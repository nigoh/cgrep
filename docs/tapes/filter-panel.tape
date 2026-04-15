# Filter panel demo
Output docs/gifs/filter-panel.gif

Set Shell "bash"
Set FontSize 14
Set Width 140
Set Height 35
Set Theme "Dracula"

Hide
  Type "export CONFLUENCE_URL=https://c.example.com CONFLUENCE_USER=u CONFLUENCE_TOKEN=t JIRA_URL=https://j.example.com JIRA_USER=u JIRA_TOKEN=t GERRIT_URL=https://g.example.com GERRIT_USER=u GERRIT_PASSWORD=p"
  Enter
Show

Type "cgrep"
Enter
Sleep 500ms

# Open filter panel
Tab
Sleep 800ms

# Navigate and toggle
Down
Sleep 400ms
Down
Sleep 400ms
Space
Sleep 600ms
Down
Down
Sleep 400ms
Space
Sleep 600ms

# Close
Escape
Sleep 500ms

Type "q"
