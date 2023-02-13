#!/usr/bin/env python3
#
# Copyright (c) 2017, 2022 ZettaScale Technology.
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ZettaScale zenoh team, <zenoh@zettascale.tech>
#


import os
from ament_index_python.packages import get_package_share_directory
from launch import LaunchDescription
from launch.actions import DeclareLaunchArgument
from launch.substitutions import LaunchConfiguration
from launch_ros.actions import Node


def generate_launch_description():

    outfile = LaunchConfiguration('outfile', default='/tmp/ros-montblanc.log')

    mb_param_dir = LaunchConfiguration(
        'mb_param_dir',
        default=os.path.join(
            get_package_share_directory('mont_blanc'),
            'param',
            'mont_blanc.yaml'))

    ld = LaunchDescription([

        DeclareLaunchArgument(
            'outfile',
            default_value=outfile,
            description='Connected USB port with OpenCR'),
        DeclareLaunchArgument(
            'mb_param_dir',
            default_value=mb_param_dir,
            description='Full path to turtlebot3 parameter file to load'),
        Node(
            package='nodes',
            executable='arequipa',
            parameters=[mb_param_dir],
            arguments=[outfile],
            output='screen'),
        Node(
            package='nodes',
            executable='barcelona',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='cordoba',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='delhi',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='freeport',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='geneva',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='georgetown',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='hamburg',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='hebron',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='kingston',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='lyon',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='madellin',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='monaco',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='osaka',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='ponce',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='portsmouth',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='rotterdam',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='taipei',
            parameters=[mb_param_dir],
            output='screen'),
        Node(
            package='nodes',
            executable='tripoli',
            parameters=[mb_param_dir],
            output='screen'),

        ])

    return ld