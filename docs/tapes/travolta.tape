# Travolta animation demo
Output docs/gifs/travolta.gif

Set Shell "bash"
Set FontSize 16
Set Width 100
Set Height 20
Set Theme "Dracula"

Hide
  Type "export CONFLUENCE_URL=https://c.example.com CONFLUENCE_USER=u CONFLUENCE_TOKEN=t JIRA_URL=https://j.example.com JIRA_USER=u JIRA_TOKEN=t GERRIT_URL=https://g.example.com GERRIT_USER=u GERRIT_PASSWORD=p"
  Enter
Show

Type "cgrep"
Enter
Sleep 500ms

Type "kubernetes"
Enter
Sleep 200ms
Enter
Sleep 4s

Type "q"
