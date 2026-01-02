# Contract: WIT Interfaces

This document defines the WebAssembly Component Model interfaces for Repo Weaver plugins.

## Package Definition

```wit
package weaver:api;
```

## Core Types (CHK008)

To ensure safety and proper redacting, we define a specific type for secrets.

```wit
// Safety wrapper for sensitive strings. 
// Host implementations MUST redact this type in logs.
type secret-string = string;
```

## Error Handling (CHK006)

Specific error variants allow the host to handle failures gracefully (e.g., retrying on throttle, failing on access denied).

```wit
variant secret-error {
    // The provider could not authenticate or permission was denied.
    access-denied,
    
    // The requested secret key was not found.
    not-found,
    
    // The provider is rate-limiting requests.
    throttled,
    
    // User-facing provider error message.
    other(string),
}
```

## Secrets Interface

Plugins export this interface to provide secrets to the host.

```wit
interface secrets {
    // Fetch a secret by logical key.
    get: func(key: string) -> result<secret-string, secret-error>;
}
```

## Host Capabilities: AWS Authentication (CHK007)

The host imports this interface to provide credentials *into* the plugin. This avoids the plugin needing to implement complex credential resolution chains.

```wit
interface aws-auth {
    record credentials {
        access-key: string,
        secret-key: secret-string,
        session-token: option<secret-string>,
        region: string,
    }

    // Host provides resolved AWS credentials to the guest.
    // Returns error if credentials cannot be resolved on the host.
    get-credentials: func() -> result<credentials, string>;
}
```

## World Definition

The full contract for a Secret Provider plugin.

```wit
world secret-provider {
    // Host capabilities
    import aws-auth;
    
    // Plugin exports
    export secrets;
}
```
