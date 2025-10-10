//! Affine transformation matrix for 2D pattern transformations
//!
//! Provides a 3x3 matrix for applying geometric transformations (scale, rotate, translate,
//! skew, shear) to embroidery patterns. Stored in row-major order for efficient operations.

use std::f64::consts::PI;

/// A 3x3 affine transformation matrix for 2D transformations
///
/// Stored in row-major order: [m00, m01, m02, m10, m11, m12, m20, m21, m22]
#[derive(Debug, Clone, PartialEq)]
pub struct EmbMatrix {
    /// Matrix elements in row-major order
    m: [f64; 9],
}

impl EmbMatrix {
    /// Create a new identity matrix
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_petal::matrix::EmbMatrix;
    ///
    /// let matrix = EmbMatrix::new();
    /// let point = matrix.transform_point(10.0, 20.0);
    /// assert_eq!(point, (10.0, 20.0));
    /// ```
    pub fn new() -> Self {
        Self {
            m: Self::identity_matrix(),
        }
    }

    /// Create a matrix from raw values
    pub fn from_values(m: [f64; 9]) -> Self {
        Self { m }
    }

    /// Get identity matrix values
    fn identity_matrix() -> [f64; 9] {
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
    }

    /// Reset to identity matrix
    pub fn reset(&mut self) {
        self.m = Self::identity_matrix();
    }

    /// Get the internal matrix array
    pub fn matrix(&self) -> &[f64; 9] {
        &self.m
    }

    /// Check if this matrix is the identity matrix
    ///
    /// Returns true if the matrix represents no transformation
    pub fn is_identity(&self) -> bool {
        const EPSILON: f64 = 1e-10;
        let identity = Self::identity_matrix();
        for (i, &identity_value) in identity.iter().enumerate() {
            if (self.m[i] - identity_value).abs() > EPSILON {
                return false;
            }
        }
        true
    }

    /// Post-multiply by a translation matrix
    ///
    /// # Arguments
    ///
    /// * `tx` - Translation in x direction
    /// * `ty` - Translation in y direction
    pub fn post_translate(&mut self, tx: f64, ty: f64) {
        let translate = Self::translation_matrix(tx, ty);
        self.m = Self::multiply_matrices(&self.m, &translate);
    }

    /// Post-multiply by a scale matrix
    ///
    /// # Arguments
    ///
    /// * `sx` - Scale factor in x direction
    /// * `sy` - Scale factor in y direction (defaults to sx if None)
    /// * `x` - Origin x coordinate for scaling (default 0.0)
    /// * `y` - Origin y coordinate for scaling (default 0.0)
    pub fn post_scale(&mut self, sx: f64, sy: Option<f64>, x: f64, y: f64) {
        let sy = sy.unwrap_or(sx);

        if x == 0.0 && y == 0.0 {
            let scale = Self::scale_matrix(sx, sy);
            self.m = Self::multiply_matrices(&self.m, &scale);
        } else {
            self.post_translate(x, y);
            self.post_scale(sx, Some(sy), 0.0, 0.0);
            self.post_translate(-x, -y);
        }
    }

    /// Post-multiply by a rotation matrix
    ///
    /// # Arguments
    ///
    /// * `theta` - Rotation angle in degrees
    /// * `x` - Origin x coordinate for rotation (default 0.0)
    /// * `y` - Origin y coordinate for rotation (default 0.0)
    pub fn post_rotate(&mut self, theta: f64, x: f64, y: f64) {
        if x == 0.0 && y == 0.0 {
            let rotate = Self::rotation_matrix(theta);
            self.m = Self::multiply_matrices(&self.m, &rotate);
        } else {
            self.post_translate(x, y);
            self.post_rotate(theta, 0.0, 0.0);
            self.post_translate(-x, -y);
        }
    }

    /// Transform a point using this matrix
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    ///
    /// # Returns
    ///
    /// Transformed (x, y) coordinates
    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        let m = &self.m;
        let nx = x * m[0] + y * m[3] + m[6];
        let ny = x * m[1] + y * m[4] + m[7];
        (nx, ny)
    }

    /// Apply transformation to a mutable point in-place
    ///
    /// # Arguments
    ///
    /// * `point` - Mutable reference to [x, y] array
    pub fn apply(&self, point: &mut [f64; 2]) {
        let (nx, ny) = self.transform_point(point[0], point[1]);
        point[0] = nx;
        point[1] = ny;
    }

    /// Compute the inverse of this matrix
    ///
    /// Note: If the matrix is singular (determinant is 0), this will leave the matrix unchanged
    pub fn inverse(&mut self) {
        let m = &self.m;

        // Pre-compute cofactor terms for efficiency
        let m48s75 = m[4] * m[8] - m[7] * m[5];
        let m38s56 = m[5] * m[6] - m[3] * m[8];
        let m37s46 = m[3] * m[7] - m[4] * m[6];

        // Calculate determinant using first row expansion
        let det = m[0] * m48s75 + m[1] * m38s56 + m[2] * m37s46;

        // Guard against singular matrices (determinant near zero) and non-finite values
        const EPSILON: f64 = 1e-10;
        if det.abs() < EPSILON || !det.is_finite() {
            // Matrix is singular or invalid, cannot invert - leave unchanged
            return;
        }

        let inverse_det = 1.0 / det;

        // Build inverse matrix using cofactor method
        self.m = [
            m48s75 * inverse_det,
            (m[2] * m[7] - m[1] * m[8]) * inverse_det,
            (m[1] * m[5] - m[2] * m[4]) * inverse_det,
            m38s56 * inverse_det,
            (m[0] * m[8] - m[2] * m[6]) * inverse_det,
            (m[3] * m[2] - m[0] * m[5]) * inverse_det,
            m37s46 * inverse_det,
            (m[6] * m[1] - m[0] * m[7]) * inverse_det,
            (m[0] * m[4] - m[3] * m[1]) * inverse_det,
        ];
    }

    // Helper functions for creating transformation matrices

    fn translation_matrix(tx: f64, ty: f64) -> [f64; 9] {
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, tx, ty, 1.0]
    }

    fn scale_matrix(sx: f64, sy: f64) -> [f64; 9] {
        [sx, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 1.0]
    }

    fn rotation_matrix(theta: f64) -> [f64; 9] {
        let tau = PI * 2.0;
        let theta_rad = theta * tau / 360.0; // Convert degrees to radians
        let ct = theta_rad.cos();
        let st = theta_rad.sin();
        [ct, st, 0.0, -st, ct, 0.0, 0.0, 0.0, 1.0]
    }

    fn multiply_matrices(m0: &[f64; 9], m1: &[f64; 9]) -> [f64; 9] {
        // Standard 3x3 matrix multiplication (row-major order)
        [
            m1[0] * m0[0] + m1[1] * m0[3] + m1[2] * m0[6],
            m1[0] * m0[1] + m1[1] * m0[4] + m1[2] * m0[7],
            m1[0] * m0[2] + m1[1] * m0[5] + m1[2] * m0[8],
            m1[3] * m0[0] + m1[4] * m0[3] + m1[5] * m0[6],
            m1[3] * m0[1] + m1[4] * m0[4] + m1[5] * m0[7],
            m1[3] * m0[2] + m1[4] * m0[5] + m1[5] * m0[8],
            m1[6] * m0[0] + m1[7] * m0[3] + m1[8] * m0[6],
            m1[6] * m0[1] + m1[7] * m0[4] + m1[8] * m0[7],
            m1[6] * m0[2] + m1[7] * m0[5] + m1[8] * m0[8],
        ]
    }
}

impl Default for EmbMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let matrix = EmbMatrix::new();
        assert!(matrix.is_identity());
        let (x, y) = matrix.transform_point(5.0, 10.0);
        assert_eq!((x, y), (5.0, 10.0));
    }

    #[test]
    fn test_is_identity_after_transform() {
        let matrix = EmbMatrix::new();
        assert!(matrix.is_identity());

        let mut matrix2 = EmbMatrix::new();
        matrix2.post_translate(10.0, 20.0);
        assert!(!matrix2.is_identity());

        let mut matrix3 = EmbMatrix::new();
        matrix3.post_rotate(90.0, 0.0, 0.0);
        assert!(!matrix3.is_identity());
    }

    #[test]
    fn test_translation() {
        let mut matrix = EmbMatrix::new();
        matrix.post_translate(10.0, 20.0);
        let (x, y) = matrix.transform_point(5.0, 10.0);
        assert_eq!((x, y), (15.0, 30.0));
    }

    #[test]
    fn test_scale() {
        let mut matrix = EmbMatrix::new();
        matrix.post_scale(2.0, None, 0.0, 0.0);
        let (x, y) = matrix.transform_point(5.0, 10.0);
        assert_eq!((x, y), (10.0, 20.0));
    }

    #[test]
    fn test_rotation_90_degrees() {
        let mut matrix = EmbMatrix::new();
        matrix.post_rotate(90.0, 0.0, 0.0);
        let (x, y) = matrix.transform_point(10.0, 0.0);
        assert!((x - 0.0).abs() < 1e-10);
        assert!((y - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_in_place() {
        let mut matrix = EmbMatrix::new();
        matrix.post_translate(5.0, 5.0);
        let mut point = [10.0, 20.0];
        matrix.apply(&mut point);
        assert_eq!(point, [15.0, 25.0]);
    }
}
