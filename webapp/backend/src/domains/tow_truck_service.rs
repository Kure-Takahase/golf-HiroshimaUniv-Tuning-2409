use super::dto::tow_truck::TowTruckDto;
use super::map_service::MapRepository;
use super::order_service::OrderRepository;
use crate::errors::AppError;
use crate::models::graph::Graph;
use crate::models::tow_truck::TowTruck;
use std::time::Instant;


use std::thread;
use std::sync::mpsc;


pub trait TowTruckRepository {
    async fn get_paginated_tow_trucks(
        &self,
        page: i32,
        page_size: i32,
        status: Option<String>,
        area_id: Option<i32>,
    ) -> Result<Vec<TowTruck>, AppError>;
    async fn update_location(&self, truck_id: i32, node_id: i32) -> Result<(), AppError>;
    async fn update_status(&self, truck_id: i32, status: &str) -> Result<(), AppError>;
    async fn find_tow_truck_by_id(&self, id: i32) -> Result<Option<TowTruck>, AppError>;
}

#[derive(Debug)]
pub struct TowTruckService<
    T: TowTruckRepository + std::fmt::Debug,
    U: OrderRepository + std::fmt::Debug,
    V: MapRepository + std::fmt::Debug,
> {
    tow_truck_repository: T,
    order_repository: U,
    map_repository: V,
}

impl<
        T: TowTruckRepository + std::fmt::Debug,
        U: OrderRepository + std::fmt::Debug,
        V: MapRepository + std::fmt::Debug,
    > TowTruckService<T, U, V>
{
    pub fn new(tow_truck_repository: T, order_repository: U, map_repository: V) -> Self {
        TowTruckService {
            tow_truck_repository,
            order_repository,
            map_repository,
        }
    }

    pub async fn get_tow_truck_by_id(&self, id: i32) -> Result<Option<TowTruckDto>, AppError> {
        let tow_truck = self.tow_truck_repository.find_tow_truck_by_id(id).await?;
        Ok(tow_truck.map(TowTruckDto::from_entity))
    }

    pub async fn get_all_tow_trucks(
        &self,
        page: i32,
        page_size: i32,
        status: Option<String>,
        area: Option<i32>,
    ) -> Result<Vec<TowTruckDto>, AppError> {
        let tow_trucks = self
            .tow_truck_repository
            .get_paginated_tow_trucks(page, page_size, status, area)
            .await?;
        let tow_truck_dtos = tow_trucks
            .into_iter()
            .map(TowTruckDto::from_entity)
            .collect();

        Ok(tow_truck_dtos)
    }

    pub async fn update_location(&self, truck_id: i32, node_id: i32) -> Result<(), AppError> {
        self.tow_truck_repository
            .update_location(truck_id, node_id)
            .await?;

        Ok(())
    }

    pub async fn get_nearest_available_tow_trucks(
        &self,
        order_id: i32,
    ) -> Result<Option<TowTruckDto>, AppError> {
        let order = self.order_repository.find_order_by_id(order_id).await?;
        let area_id = self
            .map_repository
            .get_area_id_by_node_id(order.node_id)
            .await?;
        let tow_trucks = self
            .tow_truck_repository
            .get_paginated_tow_trucks(0, -1, Some("available".to_string()), Some(area_id))
            .await?;

        let nodes = self.map_repository.get_all_nodes(Some(area_id)).await?;
        let edges = self.map_repository.get_all_edges(Some(area_id)).await?;

        let mut graph = Graph::new();
        for node in nodes {
            graph.add_node(node);
        }
        for edge in edges {
            graph.add_edge(edge);
        }

        /*
        let sorted_tow_trucks_by_distance = {
            let mut tow_trucks_with_distance: Vec<_> = tow_trucks
                .into_iter()
                .map(|truck| {
                    let distance = calculate_distance(&graph, truck.node_id, order.node_id);
                    (distance, truck)
                })
                .collect();
            println!("{:?}", tow_trucks_with_distance);

            tow_trucks_with_distance.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            tow_trucks_with_distance
        };
        */


        
        let sorted_tow_trucks_by_distance = {

            let truck_node_ids: Vec<i32> = tow_trucks.iter().map(|truck| truck.node_id).collect();
            let distances = graph.find_nearest_point(order.node_id, &truck_node_ids);

            let mut tow_trucks_with_distance: Vec<_> = distances
                .into_iter()
                .map(|(distance, node_id)| {
                    let truck = tow_trucks.iter().find(|truck| truck.node_id == node_id).unwrap().clone();
                    (distance, truck)
                })
                .collect();


            /*
            let mut tow_trucks_with_distance: Vec<_> = tow_trucks
                .into_iter()
                .map(|truck| {
                    //println!("distance_duration0 开始计时");
                    //let distance_start = Instant::now();
                    let distance = calculate_distance(&mut graph, truck.node_id, order.node_id);
                    //let distance_duration0 = distance_start.elapsed();
                    //println!("distance_duration0 时间间隔: {:?}", distance_duration0);
                    (distance, truck)
                })
                .collect();
            //println!("{:?}", tow_trucks_with_distance);
            */

            if let Some(min_truck) = tow_trucks_with_distance.iter().min_by(|a, b| a.0.partial_cmp(&b.0).unwrap()).cloned() {
                // 移除最小元素并将其放在前面
                tow_trucks_with_distance.retain(|x| !(x.0 == min_truck.0 && x.1.node_id == min_truck.1.node_id));
                let mut sorted_trucks = vec![min_truck];
                sorted_trucks.extend(tow_trucks_with_distance);
                sorted_trucks
            } else {
                // 如果没有元素，直接返回空向量
                vec![]
            }
        };
        
        //println!("sorted_tow_trucks_by_distance[0] : {}",sorted_tow_trucks_by_distance[0]);
        //println!("sorted_tow_trucks_by_distance[0].0 : {}",sorted_tow_trucks_by_distance[0].0);
        //println!("sorted_tow_trucks_by_distance[1].0 : {}",sorted_tow_trucks_by_distance[0].0);

        if sorted_tow_trucks_by_distance.is_empty() || sorted_tow_trucks_by_distance[0].0 > 10000000
        {
            return Ok(None);
        }

        let sorted_tow_truck_dtos: Vec<TowTruckDto> = sorted_tow_trucks_by_distance
            .into_iter()
            .map(|(_, truck)| TowTruckDto::from_entity(truck))
            .collect();

        //println!("sorted_tow_truck_dtos.first() : {}",sorted_tow_truck_dtos.first());
        Ok(sorted_tow_truck_dtos.first().cloned())
    }
}

fn calculate_distance(graph: &mut Graph, node_id_1: i32, node_id_2: i32) -> i32 {
    graph.shortest_path(node_id_1, node_id_2)
}
