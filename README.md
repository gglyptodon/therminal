# therminal   
[![Rust](https://github.com/gglyptodon/therminal/actions/workflows/rust.yml/badge.svg)](https://github.com/gglyptodon/therminal/actions/workflows/rust.yml)

```
therminal 

USAGE:
    therminal [OPTIONS]

OPTIONS:
    -h, --help                  Print help information
    -r, --refresh <SEC>         read sensor values again after SEC seconds [default: 30]
    -s, --sensor-id <SENSOR>    
    -t, --threshold <C>         
        --tui                   Run with UI
```

Examples:

```
therminal --tui -r 20
```
![therminal tui](https://github.com/gglyptodon/therminal/blob/main/img/therminal.png?raw=true)
