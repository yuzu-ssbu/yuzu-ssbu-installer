<template>
    <div class="column has-padding">
        <h4 class="subtitle" v-if="!repair">{{ $t('select_packages.title') }}</h4>
        <h4 class="subtitle" v-if="repair">{{ $t('select_packages.title_repair') }}</h4>

        <!-- Build options -->
        <div class="tile is-ancestor">
            <div class="tile is-parent selection-box" v-for="Lpackage in $root.$data.config.packages" :key="Lpackage.name" :index="Lpackage.name">
                <div class="tile is-child test">
                    <div class="box clickable-box" v-on:click="clicked_box(Lpackage)">
                      <div class="ribbon" v-if="Lpackage.is_new"><span>New!</span></div>
                      <label class="checkbox">
                          <b-checkbox v-model="Lpackage.default">
                            {{ Lpackage.name }}
                          </b-checkbox>
                          <span v-if="Lpackage.installed"><i>{{ $t('select_packages.installed') }}</i></span>
                      </label>
                      <div>
                        <img class="package-icon" :src="`${publicPath + Lpackage.icon}`"/>
                        <p style="padding-top: 4px;" class="package-description">
                          {{ Lpackage.description }}
                        </p>
                        <p class="package-description">
                          {{ get_extended_description(Lpackage) }}
                        </p>
                      </div>
                      <div class="version-dropdown" v-if="Lpackage.source.allow_version_control">
                        <b-dropdown class="is-right" @click.native.stop hoverable @change="(v) => select_version(Lpackage, v)" aria-role="list" scrollable max-height="300">
                            <button class="button is-primary is-small" slot="trigger">
                                <span>{{ Lpackage.selected_version.name }}</span>
                                <b-icon icon="menu-down"></b-icon>
                            </button>
                            <b-dropdown-item v-for="v in $root.package_versions[Lpackage.name]" v-bind:key="v.version" v-bind:value="{version: v.version, name: v.name}" aria-role="listitem">
                              {{ v.name }}
                            </b-dropdown-item>
                        </b-dropdown>
                    </div>
                    </div>
                </div>
            </div>
        </div>
        <div class="tile is-child is-6 box clickable-box" v-if="!$root.$data.metadata.preexisting_install"  v-on:click="installDesktopShortcut = !installDesktopShortcut">
          <h4>{{ $t('select_packages.options') }}</h4>
          <b-checkbox v-model="installDesktopShortcut" v-if="$root.$data.metadata.is_windows">
            {{ $t('select_packages.option_shortcut') }}
          </b-checkbox>
          <b-checkbox v-model="installDesktopShortcut" v-if="!$root.$data.metadata.is_windows">
            Create Shortcut
          </b-checkbox>
        </div>

        <div class="subtitle is-6" v-if="!$root.$data.metadata.preexisting_install && advanced">{{ $t('select_packages.location') }}</div>
        <div class="field has-addons" v-if="!$root.$data.metadata.preexisting_install && advanced">
            <div class="control is-expanded">
                <input class="input" type="text" v-model="$root.$data.install_location"
                        :placeholder="$t('select_packages.location_placeholder')">
            </div>
            <div class="control">
                <b-button class="is-link is-info" v-on:click="select_file">
                    {{ $t('select_packages.select') }}
                </b-button>
            </div>
        </div>

        <div class="is-right-floating is-bottom-floating">
            <div class="field is-grouped">
                <p class="control">
                    <b-button class="is-primary is-medium" v-if="!$root.$data.config.hide_advanced && !$root.$data.metadata.preexisting_install && !advanced"
                        v-on:click="advanced = true">{{ $t('select_packages.advanced') }}</b-button>
                </p>
                <p class="control">
                    <b-button class="is-primary is-medium" v-if="!$root.$data.metadata.preexisting_install"
                        v-on:click="install">{{ $t('select_packages.install') }}</b-button>
                </p>
                <p class="control">
                    <a class="button is-primary is-medium" v-if="$root.$data.metadata.preexisting_install"
                        v-on:click="install">{{ repair ? $t('select_packages.repair') : $t('select_packages.modify') }}</a>
                </p>
            </div>
        </div>

        <div class="field is-grouped is-left-floating is-bottom-floating">
            <p class="control">
                <b-button class="is-dark is-medium" v-if="$root.$data.metadata.preexisting_install"
                    v-on:click="go_back">{{ $t('back') }}</b-button>
            </p>
        </div>
    </div>
</template>

<script>
export default {
  name: 'SelectPackages',
  data: function () {
    return {
      publicPath: process.env.BASE_URL,
      advanced: false,
      repair: false,
      installDesktopShortcut: true
    }
  },
  created: function () {
    this.set_package_versions()
  },
  mounted: function () {
    this.repair = this.$route.params.repair
    // EA
    // If they are authorized, make the packages that require authorization default
    // and also deselect any packages that don't use authorization
    if (this.$root.$data.has_reward_tier) {
      for (let package_index = 0; package_index < this.$root.config.packages.length; package_index++) {
        const current_package = this.$root.config.packages[package_index]
        current_package.default = current_package.requires_authorization
      }
    }
  },
  methods: {
    set_package_versions: function () {
      for (let i = 0; i < this.$root.$data.config.packages.length; i++) {
        const pkg = this.$root.$data.config.packages[i]
        const versions = this.$root.package_versions[pkg.name]
        this.$set(pkg, 'selected_version', { version: versions[0].version, name: versions[0].name })
        const is_latest = (this.$root.package_preexisting_data[pkg.name] && this.$root.package_preexisting_data[pkg.name].latest)
        if (!is_latest && this.$root.package_preexisting_data[pkg.name] !== undefined) {
          const prev_ver = this.$root.package_preexisting_data[pkg.name].version
          for (let j = 0; j < versions.length; j++) {
            if (versions[j].version === prev_ver) {
              this.$set(pkg, 'selected_version', { version: prev_ver, name: versions[j].name })
            }
          }
        }
      }
    },
    select_file: function () {
      const that = this
      window.rpc.call('SelectInstallDir').then(function (name) {
        if (name) {
          that.$root.$data.install_location = name
        }
      })
    },
    show_overwrite_dialog: function (confirmCallback) {
      this.$buefy.dialog.confirm({
        title: this.$t('select_packages.overwriting'),
        message: this.$t('select_packages.overwriting_warning', { path: this.$root.$data.install_location }),
        confirmText: this.$t('continue'),
        cancelText: this.$t('cancel'),
        type: 'is-danger',
        hasIcon: true,
        onConfirm: confirmCallback
      })
    },
    show_nothing_picked_dialog: function () {
      this.$buefy.dialog.alert({
        title: this.$t('select_packages.nothing_picked'),
        message: this.$t('select_packages.nothing_picked_warning', { path: this.$root.$data.install_location }),
        confirmText: this.$t('cancel'),
        type: 'is-danger',
        hasIcon: true
      })
    },
    install: function () {
      if (!this.$root.config.packages.some(function (x) { return x.default })) {
        this.show_nothing_picked_dialog()
        return
      }
      // maintenance + repair
      if (this.repair) {
        this.$router.push('/install/repair/' + this.installDesktopShortcut.toString())
        return
      }
      // maintenance + modify
      if (this.$root.$data.metadata.preexisting_install) {
        this.$router.push('/install/regular/' + this.installDesktopShortcut.toString())
        return
      }
      const my = this
      this.$http.post('/api/verify-path', `path=${this.$root.$data.install_location}`).then(function (resp) {
        const data = resp.data || {}
        if (!data.exists) {
          my.$router.push('/install/regular/' + my.installDesktopShortcut.toString())
        } else {
          my.show_overwrite_dialog(function () {
            my.$router.push('/install/repair/' + my.installDesktopShortcut.toString())
          })
        }
      })
    },
    go_back: function () {
      this.$router.go(-1)
    },
    show_authentication: function () {
      this.$router.push('/authentication')
    },
    show_authorization: function () {
      this.$router.push('/authentication')
    },
    installable: function (pkg) {
      return !pkg.requires_authorization || (pkg.requires_authorization && this.$root.$data.has_reward_tier)
    },
    clicked_box: function (pkg) {
      if (this.installable(pkg)) {
        pkg.default = !pkg.default
      } else if (pkg.requires_authorization && !this.$root.$data.is_authenticated) {
        this.show_authentication()
      } else if (pkg.requires_authorization && !this.$root.$data.is_linked) {
        this.show_authorization()
      } else if (pkg.requires_authorization && !this.$root.$data.is_subscribed) {
        this.show_authorization()
      } else { // need_reward_tier_description
        this.show_authorization()
      }
    },
    get_extended_description: function (pkg) {
      if (!pkg.extended_description) {
        return ''
      }
      if (this.installable(pkg)) {
        return pkg.extended_description.no_action_description
      } else if (pkg.requires_authorization && !this.$root.$data.is_authenticated) {
        return pkg.extended_description.need_authentication_description
      } else if (pkg.requires_authorization && !this.$root.$data.is_linked) {
        return pkg.extended_description.need_link_description
      } else if (pkg.requires_authorization && !this.$root.$data.is_subscribed) {
        return pkg.extended_description.need_subscription_description
      } else { // need_reward_tier_description
        return pkg.extended_description.need_reward_tier_description
      }
    },
    select_version: function (pkg, version) {
      this.$set(pkg, 'selected_version', version)
    }
  }
}
</script>

<style>
.selection-box {
  cursor: pointer;
}
.box.clickable-box {
  position: relative;
}
.version-dropdown {
  position: absolute;
  bottom: 10px;
  right: 10px;
}
</style>
