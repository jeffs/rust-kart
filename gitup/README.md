# Gitup

The `gitup` command pulls the trunk branch of the repo whence it is called, then
deletes any local branches that are behind that trunk.

The `since` command lists commmits reachable from HEAD, but not from a specified
base branch (which defaults to the local trunk).

## To Do

* Support branch-specific trunks, which may belong to different upstream repos.
