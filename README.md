# Setting Default Flash CLI Command - Documentation

## Introduction

This documentation guides you through the process of setting the default flash CLI command using the `config.toml` file. This feature allows for a more dynamic configuration by incorporating the `{port_name}` placeholder, which is automatically replaced with the designated port name.

## Instructions

Follow these steps to configure the default flash CLI command:

### Step 1: Locate the `config.toml` file

Navigate to the directory where the `config.toml` file is located. This file typically contains various configuration settings for your application.

### Step 2: Edit the `config.toml` file

Open the `config.toml` file using a text editor of your choice.

### Step 3: Update the Flash CLI Command

Locate the section in the `config.toml` file related to the Flash CLI command. It may look something like this:

```toml
[flash]
command = "flash COM4 -f firmware.bin"
```

### Step 4: Include `{port_name}` Placeholder

Modify the Flash CLI command to include the `{port_name}` placeholder. For example:

```toml
[flash]
command = "flash {port_name} -f firmware.bin"
```

By including `{port_name}`, you enable the automatic replacement of this placeholder with the port name you've specified when sending the initial keys.

### Step 5: Save the Changes

Save the changes to the `config.toml` file.

## Conclusion

Congratulations! You have successfully set the default Flash CLI command with dynamic port configuration using the `{port_name}` placeholder in the `config.toml` file.

## Contact Information

- Twitter: [@BitWeaver](https://twitter.com/BitWeaver)
- LinkedIn: [BitWeaver](https://www.linkedin.com/in/bitweaver/)

For further assistance, questions, or collaboration opportunities, feel free to reach out to us on Twitter or LinkedIn.
