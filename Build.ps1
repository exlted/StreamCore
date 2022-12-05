function _BuildDocker {
    $ToBuild = Get-Childitem -Recurse -Filter "Dockerfile"
    foreach($file in $ToBuild) {
        Push-Location $file.Directory
        $ContainerName = "streamcore-" + $file.Directory.Name;
        $ContainerName = $ContainerName.toLower()
        docker build -t $containerName -f Dockerfile .
        Pop-Location
    }
}

function _BuildOther {

}

function Build {
    _BuildDocker
}
