# Notes on RE of the dirigera API

## Endpoints

### Devices/all

Interesting error when sending an improper patch request:

```
{
  "error": "Error",
  "message": "patchFragments.forEach is not a function"
}
```

Seems to be feeding the raw error back into the request, also appears to be JS code?
