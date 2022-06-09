/**
* JetBrains Space Automation
* This Kotlin-script file lets you automate build activities
* For more info, see https://www.jetbrains.com/help/space/automation.html
*/

job("Testing") {
    container(displayName = "Setup and Run tests", image = "ubuntu") {
    	shellScript {
    	  interpreter = "/bin/bash"
        content = """
          apt-get update && apt-get upgrade
          apt-get install -y git clang curl libssl-dev llvm libudev-dev
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source ~/.cargo/env
          rustup default stable
          rustup update nightly
          rustup update stable
          rustup target add wasm32-unknown-unknown --toolchain nightly
          cargo test --release
        """
      }
    }

    startOn {
        gitPush {
            branchFilter {
                +"refs/heads/master"
            }
        }
        codeReviewOpened{}
    }
}
