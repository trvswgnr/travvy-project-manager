$user = "trvswgnr"
$repo = "travvy-project-manager"
$latest_release = Invoke-RestMethod -Uri "https://api.github.com/repos/$user/$repo/releases/latest"
$latest_version = $latest_release.tag_name
$url = "https://github.com/$user/$repo/releases/download/$latest_version/tpm-x86_64-pc-windows-msvc.tar.gz"
Invoke-WebRequest -Uri $url -OutFile "tpm-x86_64-pc-windows-msvc.tar.gz"
tar -xzvf "tpm-x86_64-pc-windows-msvc.tar.gz"
Move-Item -Path ".\tpm.exe" -Destination "C:\Program Files\"
