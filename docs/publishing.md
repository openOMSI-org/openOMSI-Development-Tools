# Publishing and signing

1. Make a signing key once: `oopc keygen`. Keep the secret `.oopkey` private; share the
   `.oopkey.pub` so players can verify your work.
2. Build signed: `oopc build --sign <your.oopkey>`.
3. Distribute the single `.oop` file. Players drop it into openOMSI's `plugins` folder.
4. The game shows **signed by `<fingerprint>`**. If you publish your public key's fingerprint
   somewhere players trust, they can confirm a plugin is really yours and has not been changed.

`oopc verify <file.oop> --key <your.oopkey.pub>` checks a built file before you upload it.
