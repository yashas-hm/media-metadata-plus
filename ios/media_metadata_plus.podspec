Pod::Spec.new do |s|
  s.name             = 'media_metadata_plus'
  s.version          = '1.4.1'
  s.summary          = 'Read media metadata from JPEG, HEIC, MP4, MOV, PNG and WebP.'
  s.homepage         = 'https://github.com/yashas-hm/media-metadata-plus'
  s.license          = { :type => 'MIT', :file => '../LICENSE' }
  s.author           = { 'Yashas H Majmudar' => 'yashashm.dev@gmail.com' }

  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'

  s.platform         = :ios, '14.0'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386',
  }

  s.swift_version        = '5.0'
  s.dependency           'Flutter'
  s.vendored_frameworks  = 'Frameworks/media_metadata_plus.xcframework'

  # The xcframework is too large to ship in the pub.dev tarball.
  # Download it from the GitHub release at pod install time.
  s.prepare_command = <<-CMD
    mkdir -p Frameworks
    curl -fsSL "https://github.com/yashas-hm/media-metadata-plus/releases/download/v#{s.version}/ios_v#{s.version}.xcframework.zip" \
      -o /tmp/mmp_ios.xcframework.zip
    unzip -o /tmp/mmp_ios.xcframework.zip -d Frameworks
    rm /tmp/mmp_ios.xcframework.zip
  CMD

  s.resource_bundles = {'media_metadata_plus_privacy' => ['Resources/PrivacyInfo.xcprivacy']}
end
