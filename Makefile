.PHONY: clean zenoh_flow ros ros2 clean_ros clean_zf

all: zenoh_flow ros ros2

zenoh_flow:
	RUSTFLAGS='-C target-cpu=native'  cargo build --release --all-targets

ros:
	bash -c "source /opt/ros/noetic/setup.bash && cd comparison/ros/eval-ws && catkin_make"

ros2:
	bash -c "source /opt/ros/galactic/setup.bash && cd comparison/ros2/eval-ws && colcon build"


clean_ros:
	rm -rf comparison/ros/eval-ws/build/
	rm -rf comparison/ros/eval-ws/devel/

clean_ros2:
	rm -rf comparison/ros2/eval-ws/build/
	rm -rf comparison/ros2/eval-ws/install/
	rm -rf comparison/ros2/eval-ws/log/

clean_zf:
	cargo clean

clean: clean_ros clean_ros2 clean_zf

