# Parallel search progress demo (focus on status bar)
Output docs/gifs/parallel-search.gif

Set Shell "bash"
Set FontSize 14
Set Width 160
Set Height 12
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
Sleep 300ms
Enter
Sleep 4s

Type "q"
