# Help screen demo
Output docs/gifs/help.gif

Set Shell "bash"
Set FontSize 14
Set Width 140
Set Height 40
Set Theme "Dracula"

Hide
  Type "export CONFLUENCE_URL=https://c.example.com CONFLUENCE_USER=u CONFLUENCE_TOKEN=t JIRA_URL=https://j.example.com JIRA_USER=u JIRA_TOKEN=t GERRIT_URL=https://g.example.com GERRIT_USER=u GERRIT_PASSWORD=p"
  Enter
Show

Type "cgrep"
Enter
Sleep 500ms

# Open help
Type "?"
Sleep 2s

# Close help
Type "?"
Sleep 500ms

Type "q"
