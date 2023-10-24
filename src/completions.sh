__tpm() {
    local cur
    local prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    case ${COMP_CWORD} in
    1)
        COMPREPLY=($(compgen -W "open add edit delete new" -- ${cur}))
        ;;
    2)
        case ${prev} in
        open | edit | delete)
            COMPREPLY=($(compgen -W "$(cat {%config_dir%}/project_names.txt)" -- ${cur}))
            ;;
        *)
            ;;
        esac
        ;;
    esac
}

complete -F __tpm {%app_name%}
