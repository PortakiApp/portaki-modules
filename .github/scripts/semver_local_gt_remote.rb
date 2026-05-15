#!/usr/bin/env ruby
# frozen_string_literal: true

# Usage: semver_local_gt_remote.rb <local_version> <remote_version>
# Exit 0 if local is strictly greater than remote (Gem::Version ordering), else 1. Exit 2 on invalid local.

require "rubygems/version"

local = ARGV[0]
remote = ARGV[1].to_s.empty? ? "0.0.0" : ARGV[1]

begin
  a = Gem::Version.new(local)
  b = Gem::Version.new(remote)
rescue ArgumentError
  warn("invalid version: local=#{local.inspect} remote=#{remote.inspect}")
  exit(2)
end

exit(a > b ? 0 : 1)
