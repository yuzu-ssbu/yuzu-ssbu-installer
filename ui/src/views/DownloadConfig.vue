<template>
    <div class="column has-padding">
            <h4 class="subtitle">{{ $t('download_config.download_config') }}</h4>

            <br />
            <progress class="progress is-info is-medium" max="100">
                0%
            </progress>
    </div>
</template>

<script>
export default {
  name: 'DownloadConfig',
  created: function () {
    this.$root.get_and_cache_package_metadata(this.download_config)
  },
  methods: {
    download_config: function () {
      const that = this
      this.$http.get('/api/config').then(function (resp) {
        that.$root.config = resp.data
        if (!that.$root.metadata.is_launcher) {
          that.$root.get_and_cache_package_versions()
        }
        that.$root.check_authentication(that.choose_next_state, that.choose_next_state)
      }).catch(function (e) {
        console.error('Got error while downloading config: ' +
                    e)

        if (that.$root.metadata.is_launcher) {
          // Just launch the target application
          that.$root.exit()
        } else {
          that.$router.replace({
            name: 'showerr',
            params: { msg: that.$i18n.t('download_config.error_download_config', { msg: e }) }
          })
        }
      })
    },
    choose_next_state: function () {
      const app = this.$root
      // Update the updater if needed
      if (app.config.new_tool) {
        this.$router.push('/install/updater/false')
        return
      }

      if (app.metadata.preexisting_install) {
        app.install_location = app.metadata.install_path

        // Copy over installed packages
        for (let x = 0; x < app.config.packages.length; x++) {
          app.config.packages[x].default = false
          app.config.packages[x].installed = false
        }

        for (let i = 0; i < app.metadata.database.packages.length; i++) {
          // Find this config package
          for (let x = 0; x < app.config.packages.length; x++) {
            if (app.config.packages[x].name === app.metadata.database.packages[i].name) {
              app.config.packages[x].default = true
              app.config.packages[x].installed = true
            }
          }
        }

        this.$router.replace({
          name: 'migrate',
          params: { next: app.metadata.is_launcher ? '/install/regular/false' : '/modify' }
        })
      } else {
        for (let x = 0; x < app.config.packages.length; x++) {
          app.config.packages[x].installed = false
        }

        // Need to do a bit more digging to get at the
        // install location.
        this.$http.get('/api/default-path').then(function (resp) {
          if (resp.data.path != null) {
            app.install_location = resp.data.path
          }
        })

        this.$router.replace({
          name: 'migrate',
          params: { next: '/packages' }
        })
      }
    }
  }
}
</script>
