# Tabs demo
Output docs/gifs/tabs.gif

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

# New tab
Ctrl+t
Sleep 600ms

# New tab again
Ctrl+t
Sleep 600ms

# Switch left
Ctrl+Left
Sleep 600ms
Ctrl+Left
Sleep 600ms

# Switch right
Ctrl+Right
Sleep 600ms

# Close a tab
Ctrl+w
Sleep 600ms

# Jump to tab 1 with Alt+1
Alt+1
Sleep 600ms

Type "q"
