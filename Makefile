all:
	RUSTFLAGS='-C target-cpu=native'  cargo build --release --all-targets
	bash -c "source /opt/ros/noetic/setup.bash && cd comparison/ros/eval-ws && catkin_make"
	bash -c "source /opt/ros/galactic/setup.bash && cd comparison/ros2/eval-ws && colcon build"

clean:
	cargo clean
	rm -rf comparison/ros/eval-ws/build/
	rm -rf comparison/ros/eval-ws/devel/
	rm -rf comparison/ros2/eval-ws/build/
	rm -rf comparison/ros2/eval-ws/install/
	rm -rf comparison/ros2/eval-ws/log/
